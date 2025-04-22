#[allow(warnings)]
mod bindings;

use alloy_primitives::{Address, Uint};
use alloy_sol_types::{sol, SolCall, SolValue};
use bindings::{
    export,
    wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent},
    Guest, TriggerAction,
};
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

    interface IStrategyManager {
        function burnShares(
            address strategy
        ) external;
    }
}
use IStrategyManager::burnSharesCall;

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

                // Convert contract_address to Address
                let strategy_manager = Address::from_slice(&contract_address.raw_bytes);

                // Return ABI encoded payload
                let payload = TransactionPayload {
                    to: strategy_manager,
                    value: Uint::<256, 4>::ZERO,
                    data: burnSharesCall { strategy }.abi_encode().into(),
                };

                Ok(Some(payload.abi_encode().into()))
            }
            _ => Err("Unsupported trigger data".to_string()),
        }
    }
}

export!(Component with_types_in bindings);
