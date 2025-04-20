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
use sol_interfaces::TransactionPayload;
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
    // Get the DAO context with all our configuration
    let context = DaoContext::default();

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

    // Format the system prompt using the context
    let system_prompt = context.format_system_prompt();

    // Create LLM client with the configuration from context
    let client = LLMClient::with_config(&context.model, context.llm_config.clone())
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
