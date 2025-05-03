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
            // Load context
            let context = Context::load().await?;

            // Create LLM client
            let client = LLMClient::with_config(&context.model, context.llm_config.clone())
                .map_err(|e| format!("Failed to create LLM client: {}", e))?;

            // Process prompt using LLM with tools
            let llm_response = client.process_prompt(&prompt, &context, None, None).await?;

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
