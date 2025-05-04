mod bindings;
pub mod context;

use alloy_sol_types::SolType;
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use context::DaoContext;
use serde_json;
use wavs_llm::{process_prompt_with_client, types::LlmResponse};
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
            TriggerData::Raw(data) => {
                let prompt = std::str::from_utf8(&data)
                    .map_err(|e| format!("Failed to decode prompt from bytes: {}", e))?;
                Ok(prompt.to_string())
            }
            _ => Err("Unsupported trigger data".to_string()),
        }?;

        return block_on(async move {
            // Get the DAO context with all our configuration
            let context = DaoContext::load().await?;
            let llm_context = context.llm_context.clone();

            // Create LLM client implementation
            let llm_client_impl = context.create_llm_client_impl();

            // Use the helper function to process the prompt
            let result = process_prompt_with_client(
                &llm_client_impl,
                llm_context.model.clone(),
                prompt,
                llm_context,
                None,
                None,
            )
            .map_err(|e| e.to_string())?;

            // Handle the response
            match result {
                LlmResponse::Transaction(tx) => {
                    println!("Transaction to execute: {:?}", tx);

                    // Serialize transaction for execution
                    let payload = serde_json::to_vec(&tx)
                        .map_err(|e| format!("Failed to serialize transaction: {}", e))?;
                    println!("Payload: {:?}", payload);

                    Ok(Some(payload))
                }
                LlmResponse::Text(text) => {
                    println!("LLM response: {}", text);
                    Ok(None)
                }
            }
        });
    }
}

export!(Component with_types_in bindings);
