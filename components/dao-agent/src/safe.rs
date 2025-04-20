use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::SolCall;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// Import the IERC20 interface and TransactionPayload definition
use crate::sol_interfaces::{TransactionPayload, IERC20};

/// Represents a contract function call
#[derive(Serialize, Deserialize, Debug)]
pub struct ContractCall {
    pub function: String,
    pub args: Vec<serde_json::Value>,
}

/// Represents a transaction to be executed through the Safe
#[derive(Serialize, Deserialize, Debug)]
pub struct SafeTransaction {
    pub to: String,
    pub value: String, // Using string to handle large numbers safely
    #[serde(default = "default_contract_call")]
    pub contract_call: Option<ContractCall>, // JSON representation of the call to encode
    pub data: String,  // Will be populated after encoding
    pub description: String, // LLM's explanation of the transaction
}

/// Default function for ContractCall
pub fn default_contract_call() -> Option<ContractCall> {
    None
}

/// Helper function to create a TransactionPayload from a SafeTransaction
pub fn create_payload_from_safe_tx(tx: &SafeTransaction) -> Result<TransactionPayload, String> {
    // Parse address
    let to: Address = tx.to.parse().map_err(|e| format!("Invalid address: {}", e))?;

    // Parse value
    let value = U256::from_str(&tx.value).map_err(|e| format!("Invalid value: {}", e))?;

    // Handle contract calls
    let data = if let Some(contract_call) = &tx.contract_call {
        match contract_call.function.as_str() {
            "transfer" => {
                let recipient = contract_call.args[0]
                    .as_str()
                    .ok_or("Missing recipient")?
                    .parse::<Address>()
                    .map_err(|e| format!("Invalid recipient address: {}", e))?;
                let amount =
                    U256::from_str(contract_call.args[1].as_str().ok_or("Missing amount")?)
                        .map_err(|e| format!("Invalid amount: {}", e))?;

                let call = IERC20::transferCall { recipient, amount };
                Bytes::from(call.abi_encode())
            }
            _ => Bytes::default(),
        }
    } else {
        Bytes::default()
    };

    Ok(TransactionPayload { to, value, data })
}

/// Helper functions for safe operations
pub mod operations {
    use super::*;

    /// Validate a Safe transaction
    pub fn validate_transaction(tx: &SafeTransaction) -> Result<(), String> {
        // Basic validation
        if tx.to.len() != 42 || !tx.to.starts_with("0x") {
            return Err("Invalid destination address".to_string());
        }

        // Ensure value is a valid number
        if let Err(e) = U256::from_str(&tx.value) {
            return Err(format!("Invalid value: {}", e));
        }

        // If there's a contract call, validate its arguments
        if let Some(contract_call) = &tx.contract_call {
            match contract_call.function.as_str() {
                "transfer" => {
                    // Validate recipient
                    if contract_call.args.len() < 2 {
                        return Err("Transfer requires recipient and amount".to_string());
                    }

                    // Check recipient format
                    let recipient =
                        contract_call.args[0].as_str().ok_or("Recipient must be a string")?;

                    if recipient.len() != 42 || !recipient.starts_with("0x") {
                        return Err("Invalid recipient address format".to_string());
                    }

                    // Check amount format
                    let amount = contract_call.args[1].as_str().ok_or("Amount must be a string")?;

                    if U256::from_str(amount).is_err() {
                        return Err("Invalid amount format".to_string());
                    }
                }
                _ => {
                    // General validation for other function calls
                }
            }
        }

        Ok(())
    }
}
