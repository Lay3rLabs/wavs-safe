mod bindings;
mod context;
mod llm;
mod models;
mod tools;

use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolCall, SolType, SolValue};
use anyhow::Result;
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use llm::LLMClient;
use models::{DaoContext, SafeTransaction};
use serde_json::json;
use std::str::FromStr;
use tools::Message;
use wstd::runtime::block_on;

// Define the Solidity interface we're working with
sol! {
    interface IERC20 {
        function transfer(address recipient, uint256 amount) external returns (bool);
    }

    #[derive(Debug)]
    struct TransactionPayload {
        address to;
        uint256 value;
        bytes data;
    }
}

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        match trigger_action.data {
            TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
                // Decode the ABI-encoded string first
                let decoded = alloy_sol_types::sol_data::String::abi_decode(&log.data, false)
                    .map_err(|e| format!("Failed to decode ABI string: {}", e))?;

                let prompt = decoded.to_string();

                return block_on(async move {
                    let response = query_llm(&prompt).await?;

                    println!("Response: {}", response);

                    // Extract tool call or return no-op if none found
                    let tool_call = match response
                        .split("<tool_call>")
                        .nth(1)
                        .and_then(|s| s.split("</tool_call>").next())
                    {
                        Some(call) => call,
                        None => {
                            // Return a no-op transaction if no tool call is found
                            let no_op = create_no_op_transaction("No action needed");
                            let payload = create_payload_from_safe_tx(&no_op)?;
                            return Ok(Some(payload.abi_encode().to_vec()));
                        }
                    };

                    let transaction: SafeTransaction = serde_json::from_str(tool_call)
                        .map_err(|e| format!("Failed to parse transaction: {}", e))?;

                    // Return no-op if "to" address is empty or invalid
                    if transaction.to.is_empty()
                        || transaction.to == "0x"
                        || transaction.to.len() < 42
                    {
                        let no_op =
                            create_no_op_transaction("Invalid or missing destination address");
                        let payload = create_payload_from_safe_tx(&no_op)?;
                        return Ok(Some(payload.abi_encode().to_vec()));
                    }

                    let payload = create_payload_from_safe_tx(&transaction)?;

                    println!("Payload: {:?}", payload);

                    Ok(Some(payload.abi_encode().to_vec()))
                });
            }
            // TriggerData::CosmosContractEvent(TriggerDataCosmosContractEvent { .. }) => {}
            // TriggerData::Raw(input) => {}
            _ => Err("Unsupported trigger data".to_string()),
        }
    }
}

fn create_no_op_transaction(reason: &str) -> SafeTransaction {
    SafeTransaction {
        to: "0x0000000000000000000000000000000000000000".to_string(),
        value: "0".to_string(),
        contract_call: None,
        data: "0x".to_string(),
        description: format!("No action taken: {}", reason),
    }
}

/// Helper function to create a TransactionPayload from a SafeTransaction
fn create_payload_from_safe_tx(tx: &SafeTransaction) -> Result<TransactionPayload, String> {
    // Parse address
    let to: Address = tx.to.parse().map_err(|e| format!("Invalid address: {}", e))?;

    // Parse value
    let value = U256::from_str(&tx.value).map_err(|e| format!("Invalid value: {}", e))?;

    // Handle contract calls
    let data = if let Some(contract_call) = &tx.contract_call {
        match contract_call.function.as_str() {
            "transfer" => {
                let recipient = contract_call.args[0]
                    .as_str()
                    .ok_or("Missing recipient")?
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid recipient address: {}", e))?;
                let amount =
                    U256::from_str(contract_call.args[1].as_str().ok_or("Missing amount")?)
                        .map_err(|e| format!("Invalid amount: {}", e))?;

                let call = IERC20::transferCall { recipient, amount };
                Bytes::from(call.abi_encode())
            }
            _ => Bytes::default(),
        }
    } else {
        Bytes::default()
    };

    Ok(TransactionPayload { to, value, data })
}

async fn query_llm(prompt: &str) -> Result<String, String> {
    let context = DaoContext::default();

    // Format contracts for the system prompt
    let contract_descriptions = context
        .contracts
        .iter()
        .map(|contract| {
            format!(
                "Contract: {}\nAddress: {}\nABI:\n{}",
                contract.name, contract.address, contract.abi
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let system_prompt = format!(
        r#"
        You are a function calling AI model with a list of smart contracts and their ABIs in <tools></tools> XML tags, a DAO agent 
        responsible for making and executing decisions through a Gnosis Safe Module.

        Current DAO Context:
        - Safe Address: {}
        - Current Balances:
        {}
        - Allowed Addresses: {}
        - DAO Mission: {}
        - Allowed Tokens: ONLY native ETH and USDC are supported. All other token requests should be rejected.

        Available Smart Contracts:
        <tools>
        {}
        </tools>

        REQUIRED JSON FIELDS - ALL MUST BE INCLUDED:
        1. "to": destination address
        2. "value": amount in wei
        3. "data": MUST be included and set to "0x" for simple transfers
        4. "description": explanation of the action
        5. "contract_call": (optional) only for token transfers

        STRICT JSON RULES:
        - NO comments in JSON
        - NO trailing commas
        - NO explanatory text inside JSON
        - ALL fields above must be present
        - The "data" field must ALWAYS be included
        - JSON must be strictly valid

        Examples:

        1. Simple ETH transfer:
        <tool_call>
        {{
            "to": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
            "value": "1000000000000000000",
            "data": "0x",
            "description": "Sending 1 ETH to specified address"
        }}
        </tool_call>

        2. USDC transfer:
        <tool_call>
        {{
            "to": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "value": "0",
            "data": "0x",
            "contract_call": {{
                "function": "transfer",
                "args": ["0x742d35Cc6634C0532925a3b844Bc454e4438f44e", "1000000"]
            }},
            "description": "Sending 1 USDC to specified address"
        }}
        </tool_call>

        3. No action needed:
        <tool_call>
        {{
            "to": "0x0000000000000000000000000000000000000000",
            "value": "0",
            "data": "0x",
            "description": "No action needed: [reason]"
        }}
        </tool_call>

        Security Guidelines:
        - Always verify addresses are in the allowed list or contract list
        - Include the contract_call field in the JSON if a contract call is needed
        - When making a smart contract call, ALWAYS use the contract address in the "to" field
        - If using an ERC20 token (such as USDC), ALWAYS use the contract address in the "to" field
        - Never approve transactions that would spend more than the current balance
        - Be extremely cautious with value transfers
        - Reject any suspicious or unclear requests
        - Don't allow transfers of amounts greater than 1 ETH
        - IMMEDIATELY REJECT any requests for tokens other than ETH or USDC with a no-op transaction
        - ALWAYS output JSON within <tool_call></tool_call> XML tags
        - ALWAYS end your response with JSON wrapped in <tool_call></tool_call> XML tags
        - NEVER wrap <tool_call></tool_call> with anything including markdown formatting
    "#,
        context.safe_address,
        context.format_balances(),
        context.allowed_addresses.join(", "),
        context.dao_description,
        contract_descriptions,
    );

    // Create LLM client with optimized settings for deterministic tool usage
    let llm_config = llm::LLMConfig::new()
        .temperature(0.0) // Deterministic generation
        .top_p(0.1) // Narrow sampling
        .seed(42) // Fixed seed for reproducibility
        .max_tokens(Some(500))
        .context_window(Some(4096));

    let client = LLMClient::with_config("llama3.1", llm_config)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Create the messages for the chat completion
    let messages = vec![Message::new_system(system_prompt), Message::new_user(prompt.to_string())];

    // Call the LLM client
    let response = client.chat_completion_text(&messages).await?;

    Ok(response)
}

export!(Component with_types_in bindings);
