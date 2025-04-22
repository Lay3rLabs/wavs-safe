#[allow(warnings)]
mod bindings;
mod contracts;
mod safe;

use alloy_sol_types::{sol, SolValue};
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
use safe::SafeTransaction;
use wavs_wasi_chain::decode_event_log_data;

sol! {
    #[derive(Debug)]
    event BurnableSharesIncreased(address strategy, uint256 shares);

    // Used to call into smart account with intended payload
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
            TriggerData::EthContractEvent(TriggerDataEthContractEvent {
                log,
                contract_address,
                chain_name,
                block_height,
            }) => {
                // Decode event
                let BurnableSharesIncreased { strategy, .. } = decode_event_log_data!(log)
                    .map_err(|e| format!("Failed to decode event log data: {}", e))?;

                // TODO: Implement the TX call to burn the shares
                let tx = SafeTransaction {
                    to: contract_address,
                    value: 0,
                    contract_call: Some(()),
                    data: vec![],
                };

                // Return ABI encoded payload
                let payload = TransactionPayload { to: contract_address, value: 0, data: vec![] };

                // let payload_bytes = abi_encode(&payload);
                unimplemented!()
            }
            _ => Err("Unsupported trigger data".to_string()),
        }
    }
}

export!(Component with_types_in bindings);
