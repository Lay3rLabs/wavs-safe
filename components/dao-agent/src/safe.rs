use crate::context::DaoContext;
use crate::contracts::{default_contract_call, ContractCall};
use alloy_primitives::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

// Import the TransactionPayload definition
use crate::sol_interfaces::TransactionPayload;

// TODO rename and consider moving to contracts.rs
/// Represents a transaction to be executed through the Safe
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub to: String,
    pub value: String, // Using string to handle large numbers safely
    #[serde(default = "default_contract_call")]
    pub contract_call: Option<ContractCall>, // JSON representation of the call to encode
    pub data: String,  // Will be populated after encoding
    pub description: String, // LLM's explanation of the transaction
}

// TODO maybe we can simplify this...
/// Helper function to create a TransactionPayload from a Transaction
pub fn create_payload_from_tx(tx: &Transaction) -> Result<TransactionPayload, String> {
    // Parse address
    let to: Address = tx.to.parse().map_err(|e| format!("Invalid address: {}", e))?;

    // Parse value
    let value = U256::from_str(&tx.value).map_err(|e| format!("Invalid value: {}", e))?;

    // Handle contract calls
    let data = if let Some(contract_call) = &tx.contract_call {
        // Get contract details from the context
        let context = DaoContext::default();

        // Try to find the contract by address
        let contract = context
            .contracts
            .iter()
            .find(|c| c.address.to_lowercase() == tx.to.to_lowercase())
            .ok_or_else(|| format!("Cannot find contract at address {}", tx.to))?;

        // Use the contract to encode the function call
        contract.encode_function_call(&contract_call.function, &contract_call.args)?
    } else {
        Bytes::default()
    };

    Ok(TransactionPayload { to, value, data })
}

/// Helper functions for safe operations
pub mod operations {
    use super::*;

    /// Validate a Safe transaction
    pub fn validate_transaction(tx: &Transaction) -> Result<(), String> {
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
            // Get context to look up contract
            let context = DaoContext::default();

            // Find the contract
            let contract = context
                .contracts
                .iter()
                .find(|c| c.address.to_lowercase() == tx.to.to_lowercase())
                .ok_or_else(|| format!("Unknown contract at address: {}", tx.to))?;

            // Validate the function call using the contract
            contract.validate_function_call(&contract_call.function, &contract_call.args)?;
        }

        Ok(())
    }
}
