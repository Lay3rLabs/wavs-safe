mod bindings;
pub mod context;

use alloy_sol_types::{SolType, SolValue};
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use context::DaoContext;
use wavs_agent::llm::LlmResponse;
use wavs_agent::{
    contracts::{create_payload_from_tx, transaction_operations::validate_transaction},
    errors::AgentError,
    llm::LLMClient,
};
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
            let llm_context = context.llm_context;

            // Create LLM client
            let client = LLMClient::with_config(&llm_context.model, llm_context.llm_config.clone())
                .map_err(|e| e.to_string())?;

            // Process prompt
            let llm_response = client
                .process_prompt(&prompt, &llm_context, None, None)
                .await
                .map_err(|e| e.to_string())?;

            // Handle the response
            match llm_response {
                LlmResponse::Transaction(tx) => {
                    println!("Transaction to execute: {:?}", tx);

                    // Validate the transaction first
                    validate_transaction(&tx).map_err(|e| {
                        AgentError::Transaction(format!("Transaction validation failed: {}", e))
                            .to_string()
                    })?;

                    // Create the transaction payload
                    let payload = create_payload_from_tx(&tx).map_err(|e| {
                        AgentError::Transaction(format!("Failed to create transaction: {}", e))
                            .to_string()
                    })?;

                    println!("Payload: {:?}", payload);

                    Ok(Some(payload.abi_encode().to_vec()))
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
