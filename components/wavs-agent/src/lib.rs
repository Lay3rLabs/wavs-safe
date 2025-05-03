mod bindings;
pub mod context;
mod contracts;
mod llm;
mod sol_interfaces;
pub mod tools;

use alloy_sol_types::{SolType, SolValue};
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use context::Context;
use llm::{LLMClient, LlmResponse};
use serde::{Deserialize, Serialize};
use wstd::runtime::block_on;

/// Input structure for the component
#[derive(Debug, Deserialize, Serialize)]
struct AgentInput {
    prompt: String,
    #[serde(default)]
    context_uri: Option<String>,
    #[serde(default)]
    context_json: Option<String>,
}

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let agent_input = match trigger_action.data {
            // Handle raw data - expected to be JSON containing prompt and optional context
            TriggerData::Raw(data) => {
                let input_str = std::str::from_utf8(&data)
                    .map_err(|e| format!("Failed to decode input from bytes: {}", e))?;

                // First try to parse as AgentInput
                match serde_json::from_str::<AgentInput>(input_str) {
                    Ok(input) => Ok(input),
                    Err(_) => {
                        // Fall back to treating the entire input as just the prompt
                        Ok(AgentInput {
                            prompt: input_str.to_string(),
                            context_uri: None,
                            context_json: None,
                        })
                    }
                }
            }
            // Handle Ethereum events - decode the string from ABI-encoded data
            TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
                // Decode the ABI-encoded string first
                let decoded = alloy_sol_types::sol_data::String::abi_decode(&log.data, false)
                    .map_err(|e| format!("Failed to decode ABI string: {}", e))?;

                let input_str = decoded.to_string();

                // Try to parse as AgentInput
                match serde_json::from_str::<AgentInput>(&input_str) {
                    Ok(input) => Ok(input),
                    Err(_) => {
                        // Fall back to treating the entire input as just the prompt
                        Ok(AgentInput { prompt: input_str, context_uri: None, context_json: None })
                    }
                }
            }
            _ => Err("Unsupported trigger data".to_string()),
        }?;

        return block_on(async move {
            // Load context - either from the environment, from a provided URI, or from inline JSON
            let context = if let Some(uri) = agent_input.context_uri {
                // Use provided URI to load context
                Context::load_from_uri(&uri).await?
            } else if let Some(json) = agent_input.context_json {
                // Parse inline JSON context
                Context::from_json(&json)?
            } else {
                // Default to environment-based context loading
                Context::load().await?
            };

            // Create LLM client
            let client = LLMClient::with_config(&context.model, context.llm_config.clone())
                .map_err(|e| format!("Failed to create LLM client: {}", e))?;

            // Process prompt using LLM with tools
            let llm_response =
                client.process_prompt(&agent_input.prompt, &context, None, None).await?;

            // Match the response type
            match llm_response {
                LlmResponse::Transaction(transaction) => {
                    // Create the transaction payload
                    let payload = contracts::create_payload_from_tx(&transaction)?;
                    println!("Payload: {:?}", payload);

                    Ok(Some(payload.abi_encode().to_vec()))
                }
                LlmResponse::Text(text) => {
                    if !text.is_empty() {
                        println!("LLM response: {}", text);
                        Ok(Some(text.as_bytes().to_vec()))
                    } else {
                        println!("No action needed");
                        Ok(None)
                    }
                }
            }
        });
    }
}

export!(Component with_types_in bindings);
