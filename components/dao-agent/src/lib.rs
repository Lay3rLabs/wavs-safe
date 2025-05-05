#[allow(warnings)]
mod bindings;
pub mod context;
pub mod sol_interfaces;

use crate::sol_interfaces::TransactionPayload;
use alloy_sol_types::SolType;
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use context::DaoContext;
use serde_json;
use wavs_llm::{client::new_client, traits::GuestLlmClientManager, types::LlmResponse};

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

        // Get the DAO context with all our configuration
        let context = DaoContext::load()?;
        let llm_context = context.llm_context.clone();

        // Create LLM client implementation using the standalone constructor
        let llm_client = new_client(llm_context.model.clone()).map_err(|e| e.to_string())?;

        // Use the helper function to process the prompt
        let result = llm_client
            .process_prompt(prompt, llm_context.clone(), None, None)
            .map_err(|e| e.to_string())?;

        // Handle the response
        match result {
            LlmResponse::Transaction(tx) => {
                println!("Transaction to execute: {:?}", tx);

                // TODO fix encoding
                // Serialize transaction for execution
                let payload = TransactionPayload { to: tx.to, value: tx.value, data: tx.data };
                println!("Payload: {:?}", payload);

                Ok(Some(payload))
            }
            LlmResponse::Text(text) => {
                println!("LLM response: {}", text);
                Ok(None)
            }
        }
    }
}

export!(Component with_types_in bindings);
