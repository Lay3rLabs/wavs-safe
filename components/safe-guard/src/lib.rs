#[allow(warnings)]
mod bindings;
use alloy_sol_types::{sol, SolValue};
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEvmContractEvent},
    Guest, TriggerAction,
};
use wavs_wasi_utils::decode_event_log_data;

use crate::bindings::WasmResponse;

sol! {
    #[derive(Debug)]
    event ApproveHash(bytes32 indexed approvedHash, address indexed owner);

    #[derive(Debug)]
    struct ValidationPayload {
        bytes32 approvedHash;
        bool approved;
    }
}

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<WasmResponse>, String> {
        match trigger_action.data {
            TriggerData::EvmContractEvent(TriggerDataEvmContractEvent { log, .. }) => {
                // Decode event
                let ApproveHash { approvedHash, .. } = decode_event_log_data!(log)
                    .map_err(|e| format!("Failed to decode event log data: {}", e))?;

                // Return true. Normally you would like to have some other logic in the component
                // to decide if the transaction should be approved or not.
                Ok(Some(WasmResponse {
                    payload: ValidationPayload { approvedHash, approved: true }.abi_encode(),
                    ordering: None,
                }))
            }
            _ => Err("Unsupported trigger data".to_string()),
        }
    }
}

export!(Component with_types_in bindings);
