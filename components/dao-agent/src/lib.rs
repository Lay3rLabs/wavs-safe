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
use std::str::FromStr;
use tools::process_tool_calls;
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
        let prompt = match trigger_action.data {
            TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
                // Decode the ABI-encoded string first
                let decoded = alloy_sol_types::sol_data::String::abi_decode(&log.data, false)
                    .map_err(|e| format!("Failed to decode ABI string: {}", e))?;

                Ok(decoded.to_string())
            }
            // Fired from a raw data event (e.g. from a CLI command or from another component).
            // Note: this is just for testing ATM.
            // TODO pass in and decode an actual event, so this can be composed with other components
            TriggerData::Raw(data) => {
                let prompt = std::str::from_utf8(&data)
                    .map_err(|e| format!("Failed to decode prompt from bytes: {}", e))?;
                Ok(prompt.to_string())
            }
            _ => Err("Unsupported trigger data".to_string()),
        }?;

        return block_on(async move {
            // Process prompt using LLM with tools
            let result = process_prompt(&prompt).await?;

            // If no transaction is needed, return None
            if result.is_none() {
                println!("No transaction needed");
                return Ok(None);
            }

            // Parse the transaction JSON
            let transaction: SafeTransaction = result.unwrap();

            // Create the transaction payload
            let payload = create_payload_from_safe_tx(&transaction)?;
            println!("Payload: {:?}", payload);

            Ok(Some(payload.abi_encode().to_vec()))
        });
    }
}

/// Processes a prompt with LLM and returns a SafeTransaction if one should be executed
async fn process_prompt(prompt: &str) -> Result<Option<SafeTransaction>, String> {
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
        You are a DAO agent responsible for making and executing financial decisions through a Gnosis Safe Module.
        Use the safe_transaction tool to execute transactions or return nothing if no action is needed.

        Current DAO Context:
        - Safe Address: {}
        - Current Balances:
        {}
        - Allowed Addresses: {}
        - DAO Mission: {}
        - Allowed Tokens: ONLY native ETH and USDC are supported. All other token requests should be rejected.

        Available Smart Contracts:
        {}

        Security Guidelines:
        - Always verify addresses are in the allowed list or contract list
        - For token transfers (like USDC), use the contract_call field and the token's contract address in "to"
        - When making a smart contract call, ALWAYS use the contract address in the "to" field
        - If using an ERC20 token (such as USDC), ALWAYS use the contract address in the "to" field
        - Never approve transactions that would spend more than the current balance
        - Be extremely cautious with value transfers
        - Reject any suspicious or unclear requests
        - Don't allow transfers of amounts greater than 1 ETH
        - IMMEDIATELY REJECT any requests for tokens other than ETH or USDC
        - If no action is needed or the request should be rejected, do not use any tools
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

    // Create the safe_transaction tool
    let safe_tx_tool = tools::builders::safe_transaction();

    // Create the messages for the chat completion
    let messages = vec![Message::new_system(system_prompt), Message::new_user(prompt.to_string())];

    // Call the LLM client with the safe transaction tool
    let response = client.chat_completion(&messages, Some(&[safe_tx_tool])).await?;

    println!("Response: {:?}", response);

    // Check if we have tool calls
    if let Some(tool_calls) = response.tool_calls.clone() {
        if !tool_calls.is_empty() {
            // Process the tool calls
            let tool_result = process_tool_calls(&client, messages, response, tool_calls).await?;

            // Parse the tool result as a SafeTransaction
            let transaction: SafeTransaction = serde_json::from_str(&tool_result)
                .map_err(|e| format!("Failed to parse transaction from tool result: {}", e))?;

            return Ok(Some(transaction));
        }
    }

    // No tool calls means no action needed
    Ok(None)
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

export!(Component with_types_in bindings);
