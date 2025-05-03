use crate::context::Context;
use crate::errors::{AgentError, AgentResult};
use crate::sol_interfaces::TransactionPayload;
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_json_abi::{Function, JsonAbi};
use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use hex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a smart contract that the DAO can interact with
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub name: String,
    pub address: String,
    pub abi: String,                 // JSON ABI string
    pub description: Option<String>, // Optional description of what the contract does
}

/// Helper methods for working with contracts
impl Contract {
    /// Create a new Contract instance
    pub fn new(name: &str, address: &str, abi: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            abi: abi.to_string(),
            description: None,
        }
    }

    /// Create a new Contract instance with description
    pub fn new_with_description(name: &str, address: &str, abi: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
            abi: abi.to_string(),
            description: Some(description.to_string()),
        }
    }

    /// Parse the JSON ABI to JsonAbi struct
    fn parse_abi(&self) -> AgentResult<JsonAbi> {
        serde_json::from_str(&self.abi)
            .map_err(|e| AgentError::Contract(format!("Failed to parse ABI: {}", e)))
    }

    /// Encode a function call for this contract using the ABI
    pub fn encode_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> AgentResult<Bytes> {
        // Find the function in the parsed ABI
        let function = self.find_function(function_name)?;

        // Get function selector
        let selector = function.selector();

        // Encode the arguments
        let encoded_args = encode_function_args(&function, args)?;

        // Combine selector and encoded args
        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded_args);

        Ok(Bytes::from(calldata))
    }

    /// Find a function in the ABI
    pub fn find_function(&self, function_name: &str) -> AgentResult<Function> {
        let json_abi = self.parse_abi()?;

        json_abi.functions().find(|f| f.name == function_name).cloned().ok_or_else(|| {
            AgentError::Contract(format!("Function '{}' not found in ABI", function_name))
        })
    }

    /// Validate function arguments against the ABI
    pub fn validate_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> AgentResult<()> {
        // Find the function in the ABI
        let function = self.find_function(function_name)?;

        // Check argument count
        if function.inputs.len() != args.len() {
            return Err(AgentError::Contract(format!(
                "Function '{}' expects {} arguments, but {} were provided",
                function_name,
                function.inputs.len(),
                args.len()
            )));
        }

        // Try encoding the arguments - if it fails, it's invalid
        encode_function_args(&function, args)?;

        Ok(())
    }
}

/// Represents a contract function call
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractCall {
    pub function: String,
    pub args: Vec<serde_json::Value>,
}

/// Default function for ContractCall
pub fn default_contract_call() -> Option<ContractCall> {
    None
}

/// Convert a string to a DynSolValue based on the type
fn json_to_sol_value(value: &serde_json::Value, ty: &DynSolType) -> AgentResult<DynSolValue> {
    match ty {
        DynSolType::Address => {
            // Convert string address to DynSolValue::Address
            let addr_str = value
                .as_str()
                .ok_or(AgentError::Contract("Address must be a string".to_string()))?;
            let address = Address::from_str(addr_str)
                .map_err(|_| AgentError::Contract(format!("Invalid address: {}", addr_str)))?;
            Ok(DynSolValue::Address(address))
        }
        DynSolType::Uint(bits) => {
            // Convert string number to DynSolValue::Uint
            let num_str = value
                .as_str()
                .ok_or(AgentError::Contract("Number must be a string".to_string()))?;
            let num = U256::from_str(num_str)
                .map_err(|_| AgentError::Contract(format!("Invalid number: {}", num_str)))?;
            Ok(DynSolValue::Uint(num, *bits))
        }
        DynSolType::Bool => {
            // Convert JSON boolean to DynSolValue::Bool
            let bool_val = value
                .as_bool()
                .ok_or(AgentError::Contract("Expected a boolean value".to_string()))?;
            Ok(DynSolValue::Bool(bool_val))
        }
        DynSolType::String => {
            // Convert JSON string to DynSolValue::String
            let string_val = value
                .as_str()
                .ok_or(AgentError::Contract("Expected a string value".to_string()))?;
            Ok(DynSolValue::String(string_val.to_string()))
        }
        DynSolType::Bytes => {
            // Convert hex string to DynSolValue::Bytes
            let bytes_str = value
                .as_str()
                .ok_or(AgentError::Contract("Bytes must be a hex string".to_string()))?;
            if !bytes_str.starts_with("0x") {
                return Err(AgentError::Contract("Bytes must start with 0x".to_string()));
            }
            let hex_str = &bytes_str[2..];
            let bytes = hex::decode(hex_str)
                .map_err(|_| AgentError::Contract("Invalid hex string".to_string()))?;
            Ok(DynSolValue::Bytes(bytes))
        }
        DynSolType::FixedBytes(size) => {
            // Convert hex string to fixed-size bytes
            let bytes_str = value
                .as_str()
                .ok_or(AgentError::Contract("Bytes must be a hex string".to_string()))?;
            if !bytes_str.starts_with("0x") {
                return Err(AgentError::Contract("Bytes must start with 0x".to_string()));
            }
            let hex_str = &bytes_str[2..];
            let bytes = hex::decode(hex_str)
                .map_err(|_| AgentError::Contract("Invalid hex string".to_string()))?;

            if bytes.len() > *size {
                return Err(AgentError::Contract(format!("Hex string too long for bytes{}", size)));
            }

            // For bytes32, create a FixedBytes<32>
            if *size == 32 {
                let mut fixed = [0u8; 32];
                let start = 32 - bytes.len();
                fixed[start..].copy_from_slice(&bytes);
                Ok(DynSolValue::FixedBytes(FixedBytes::from(fixed), 32))
            } else {
                // For other sizes, use regular bytes
                Ok(DynSolValue::Bytes(bytes))
            }
        }
        // Add handling for other types as needed
        _ => Err(AgentError::Contract(format!("Unsupported type: {:?}", ty))),
    }
}

/// Encode function arguments using Alloy's built-in functionality
fn encode_function_args(function: &Function, args: &[serde_json::Value]) -> AgentResult<Vec<u8>> {
    // If there are no arguments, return an empty vector
    if args.is_empty() {
        return Ok(Vec::new());
    }

    // Parse each parameter's type
    let param_types: Vec<DynSolType> = function
        .inputs
        .iter()
        .map(|param| {
            DynSolType::parse(&param.ty).map_err(|e| {
                AgentError::Contract(format!("Invalid parameter type '{}': {}", param.ty, e))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Convert each JSON value to a DynSolValue
    let mut values = Vec::with_capacity(args.len());

    for (i, (arg, ty)) in args.iter().zip(&param_types).enumerate() {
        match json_to_sol_value(arg, ty) {
            Ok(value) => values.push(value),
            Err(e) => {
                return Err(AgentError::Contract(format!("Error converting argument {}: {}", i, e)))
            }
        }
    }

    // Manually encode according to the ABI specification
    // First, encode head and tail parts
    let mut head = Vec::new();
    let mut tail = Vec::new();

    for (i, (value, ty)) in values.iter().zip(&param_types).enumerate() {
        if is_dynamic_type(ty) {
            // For dynamic types, the head contains the offset to the data
            let offset = head.len() + (values.len() - i) * 32; // Calculate offset
            head.extend_from_slice(&U256::from(offset).to_be_bytes::<32>());

            // The tail contains the actual data
            let encoded = value.abi_encode();
            tail.extend_from_slice(&encoded);
        } else {
            // For static types, encode directly in the head
            let encoded = value.abi_encode();
            head.extend_from_slice(&encoded);
        }
    }

    // Combine head and tail
    let mut result = Vec::new();
    result.extend_from_slice(&head);
    result.extend_from_slice(&tail);

    Ok(result)
}

/// Check if a type is dynamic according to ABI spec
fn is_dynamic_type(ty: &DynSolType) -> bool {
    matches!(
        ty,
        DynSolType::String | DynSolType::Bytes | DynSolType::Array(_) | DynSolType::Tuple(_)
    )
}

/// Represents a transaction to be executed through a wallet
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub to: String,
    pub value: String, // Using string to handle large numbers safely
    #[serde(default = "default_contract_call")]
    pub contract_call: Option<ContractCall>, // JSON representation of the call to encode
    pub data: String,  // Will be populated after encoding
    pub description: String, // LLM's explanation of the transaction
}

impl Transaction {
    /// Basic validation of transaction fields
    pub fn is_valid(&self) -> bool {
        // Check destination address format
        if self.to.len() != 42 || !self.to.starts_with("0x") {
            return false;
        }

        // Check if value is a valid number
        if U256::from_str(&self.value).is_err() {
            return false;
        }

        // Check if contract call is coherent
        if let Some(call) = &self.contract_call {
            if call.function.is_empty() {
                return false;
            }
        }

        true
    }
}

/// Helper function to create a TransactionPayload from a Transaction
pub fn create_payload_from_tx(tx: &Transaction) -> AgentResult<TransactionPayload> {
    // Parse address
    let to: Address =
        tx.to.parse().map_err(|e| AgentError::Transaction(format!("Invalid address: {}", e)))?;

    // Parse value
    let value = U256::from_str(&tx.value)
        .map_err(|e| AgentError::Transaction(format!("Invalid value: {}", e)))?;

    // Handle contract calls
    let data = if let Some(contract_call) = &tx.contract_call {
        // Get contract details from the context
        let context = Context::default();

        // Try to find the contract by address
        let contract = context
            .contracts
            .iter()
            .find(|c| c.address.to_lowercase() == tx.to.to_lowercase())
            .ok_or_else(|| {
                AgentError::Contract(format!("Cannot find contract at address {}", tx.to))
            })?;

        // Use the contract to encode the function call
        contract.encode_function_call(&contract_call.function, &contract_call.args)?
    } else {
        Bytes::default()
    };

    Ok(TransactionPayload { to, value, data })
}

/// Helper functions for transaction operations
pub mod transaction_operations {
    use super::*;

    /// Validate a transaction
    pub fn validate_transaction(tx: &Transaction) -> AgentResult<()> {
        // Basic validation
        if tx.to.len() != 42 || !tx.to.starts_with("0x") {
            return Err(AgentError::Transaction("Invalid destination address".to_string()));
        }

        // Ensure value is a valid number
        if let Err(e) = U256::from_str(&tx.value) {
            return Err(AgentError::Transaction(format!("Invalid value: {}", e)));
        }

        // Get context to look up contracts
        let context = Context::default();

        // If there's a contract call, validate its arguments
        if let Some(contract_call) = &tx.contract_call {
            // Find the contract
            let contract = context
                .contracts
                .iter()
                .find(|c| c.address.to_lowercase() == tx.to.to_lowercase())
                .ok_or_else(|| {
                    AgentError::Contract(format!("Unknown contract at address: {}", tx.to))
                })?;

            // Validate the function call using the contract
            contract.validate_function_call(&contract_call.function, &contract_call.args)?;
        }

        Ok(())
    }
}
