mod bindings;
mod context;
mod contracts;
mod llm;
mod safe;
mod sol_interfaces;
mod tools;

use alloy_sol_types::{SolType, SolValue};
use anyhow::Result;
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use context::DaoContext;
use llm::LLMClient;
use safe::SafeTransaction;
use tools::{process_tool_calls, Message};
use wstd::runtime::block_on;

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

            // Create the transaction payload using the function from safe.rs
            let payload = safe::create_payload_from_safe_tx(&transaction)?;
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

    // Create the tools for ETH transfers
    let eth_tool = tools::builders::send_eth();

    // Generate tools from smart contract ABIs
    let mut all_tools = vec![eth_tool];

    // Add contract-specific tools
    for contract in &context.contracts {
        let contract_tools = tools::builders::from_contract(contract);
        println!("Generated {} tools from {} contract", contract_tools.len(), contract.name);
        all_tools.extend(contract_tools);
    }

    // Print all available tools for debugging
    println!("Total available tools: {}", all_tools.len());
    for tool in &all_tools {
        println!(
            "Tool: {} - {}",
            tool.function.name,
            tool.function.description.as_ref().unwrap_or(&"No description".to_string())
        );
    }

    // Format the supported tokens list for the prompt
    let supported_tokens = context.get_supported_token_symbols().join(", ");

    // TODO move this to the context
    let system_prompt = format!(
        r#"
        You are a DAO agent responsible for making and executing financial decisions through a Gnosis Safe Module.
        
        You have several tools available:
        - Use the send_eth tool to send ETH to addresses
        - Use the contract_* tools to interact with smart contracts (including ERC20 tokens like USDC)
        
        Return nothing if no action is needed.

        Current DAO Context:
        - Safe Address: {}
        - Current Balances:
        {}
        - Allowed Addresses: {}
        - DAO Mission: {}
        - Allowed Tokens: ONLY native ETH and {} are supported. All other token requests should be rejected.

        Available Smart Contracts:
        {}

        Security Guidelines:
        - Always verify addresses are in the allowed list or contract list
        - For ERC20 token transfers (like USDC), use the contract_usdc_transfer tool
        - For ETH transfers, use the send_eth tool
        - For other smart contract interactions, use the matching contract_* tool
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
        supported_tokens,
        contract_descriptions,
    );

    // TODO move this to the context, parse from JSON
    // Create LLM client with optimized settings for deterministic tool usage
    let llm_config = llm::LLMConfig::new()
        .temperature(0.0) // Deterministic generation
        .top_p(0.1) // Narrow sampling
        .seed(42) // Fixed seed for reproducibility
        .max_tokens(Some(500))
        .context_window(Some(4096));

    let client = LLMClient::with_config("llama3.2", llm_config)
        .map_err(|e| format!("Failed to create LLM client: {}", e))?;

    // Create the messages for the chat completion
    let messages = vec![Message::new_system(system_prompt), Message::new_user(prompt.to_string())];

    // Call the LLM client with all tools
    let response = client.chat_completion(&messages, Some(&all_tools)).await?;

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

export!(Component with_types_in bindings);
