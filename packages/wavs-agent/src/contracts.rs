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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Bytes;
    use serde_json::json;

    #[test]
    fn test_contract_creation() {
        // Test basic constructor
        let contract = Contract::new(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
        );

        assert_eq!(contract.name, "TestContract");
        assert_eq!(contract.address, "0x1234567890123456789012345678901234567890");
        assert_eq!(
            contract.abi,
            "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]"
        );
        assert!(contract.description.is_none());

        // Test constructor with description
        let contract_with_desc = Contract::new_with_description(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            "[{\"name\":\"test\",\"type\":\"function\",\"inputs\":[],\"outputs\":[]}]",
            "Test contract description",
        );

        assert_eq!(contract_with_desc.name, "TestContract");
        assert_eq!(contract_with_desc.address, "0x1234567890123456789012345678901234567890");
        assert_eq!(contract_with_desc.description.unwrap(), "Test contract description");
    }

    #[test]
    fn test_parse_abi() {
        // Valid ABI
        let contract = Contract::new(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            r#"[{
                "name": "transfer",
                "type": "function",
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "amount", "type": "uint256"}
                ],
                "outputs": [{"name": "", "type": "bool"}]
            }]"#,
        );

        let abi_result = contract.parse_abi();
        assert!(abi_result.is_ok());
        let abi = abi_result.unwrap();

        // Check that the ABI was parsed successfully
        let functions: Vec<_> = abi.functions().collect();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "transfer");

        // Invalid ABI (malformed JSON)
        let invalid_contract = Contract::new(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            "{invalid-json",
        );

        let invalid_abi = invalid_contract.parse_abi();
        assert!(invalid_abi.is_err());
    }

    #[test]
    fn test_find_function() {
        let contract = Contract::new(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            r#"[{
                "name": "transfer",
                "type": "function",
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "amount", "type": "uint256"}
                ],
                "outputs": [{"name": "", "type": "bool"}]
            },
            {
                "name": "balanceOf",
                "type": "function",
                "inputs": [
                    {"name": "account", "type": "address"}
                ],
                "outputs": [{"name": "", "type": "uint256"}]
            }]"#,
        );

        // Find existing function
        let transfer_result = contract.find_function("transfer");
        assert!(transfer_result.is_ok());
        let transfer = transfer_result.unwrap();
        assert_eq!(transfer.name, "transfer");
        assert_eq!(transfer.inputs.len(), 2);

        // Find another existing function
        let balance_result = contract.find_function("balanceOf");
        assert!(balance_result.is_ok());
        let balance = balance_result.unwrap();
        assert_eq!(balance.name, "balanceOf");
        assert_eq!(balance.inputs.len(), 1);

        // Function not found
        let missing_result = contract.find_function("nonExistentFunction");
        assert!(missing_result.is_err());
    }

    #[test]
    fn test_validate_function_call() {
        let contract = Contract::new(
            "TestContract",
            "0x1234567890123456789012345678901234567890",
            r#"[{
                "name": "transfer",
                "type": "function",
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "amount", "type": "uint256"}
                ],
                "outputs": [{"name": "", "type": "bool"}]
            }]"#,
        );

        // Valid arguments
        let valid_args = vec![
            json!("0x1234567890123456789012345678901234567890"),
            json!("1000000000000000000"), // 1 ETH in wei
        ];
        let result = contract.validate_function_call("transfer", &valid_args);
        assert!(result.is_ok());

        // Wrong number of arguments
        let too_few_args = vec![json!("0x1234567890123456789012345678901234567890")];
        let result = contract.validate_function_call("transfer", &too_few_args);
        assert!(result.is_err());

        // Wrong argument type (e.g., invalid address)
        let invalid_args = vec![json!("not-an-address"), json!("1000000000000000000")];
        let result = contract.validate_function_call("transfer", &invalid_args);
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_is_valid() {
        // Valid transaction
        let valid_tx = Transaction {
            to: "0x1234567890123456789012345678901234567890".to_string(),
            value: "1000000000000000000".to_string(), // 1 ETH
            contract_call: Some(ContractCall {
                function: "transfer".to_string(),
                args: vec![
                    json!("0x0987654321098765432109876543210987654321"),
                    json!("500000000000000000"), // 0.5 ETH
                ],
            }),
            data: "0x".to_string(),
            description: "Test transaction".to_string(),
        };
        assert!(valid_tx.is_valid());

        // Invalid address
        let invalid_address_tx = Transaction {
            to: "invalid-address".to_string(),
            value: "1000000000000000000".to_string(),
            contract_call: None,
            data: "0x".to_string(),
            description: "Invalid address transaction".to_string(),
        };
        assert!(!invalid_address_tx.is_valid());

        // Invalid value
        let invalid_value_tx = Transaction {
            to: "0x1234567890123456789012345678901234567890".to_string(),
            value: "not-a-number".to_string(),
            contract_call: None,
            data: "0x".to_string(),
            description: "Invalid value transaction".to_string(),
        };
        assert!(!invalid_value_tx.is_valid());

        // Invalid contract call (empty function name)
        let invalid_call_tx = Transaction {
            to: "0x1234567890123456789012345678901234567890".to_string(),
            value: "0".to_string(),
            contract_call: Some(ContractCall { function: "".to_string(), args: vec![] }),
            data: "0x".to_string(),
            description: "Invalid contract call transaction".to_string(),
        };
        assert!(!invalid_call_tx.is_valid());
    }

    #[test]
    fn test_json_to_sol_value() {
        use alloy_dyn_abi::DynSolType;

        // Test address conversion
        let addr_type = DynSolType::Address;
        let addr_json = json!("0x1234567890123456789012345678901234567890");
        let addr_result = json_to_sol_value(&addr_json, &addr_type);
        assert!(addr_result.is_ok());

        // Test uint conversion
        let uint_type = DynSolType::Uint(256);
        let uint_json = json!("1000000000000000000");
        let uint_result = json_to_sol_value(&uint_json, &uint_type);
        assert!(uint_result.is_ok());

        // Test bool conversion
        let bool_type = DynSolType::Bool;
        let bool_json = json!(true);
        let bool_result = json_to_sol_value(&bool_json, &bool_type);
        assert!(bool_result.is_ok());

        // Test string conversion
        let string_type = DynSolType::String;
        let string_json = json!("test string");
        let string_result = json_to_sol_value(&string_json, &string_type);
        assert!(string_result.is_ok());

        // Test bytes conversion
        let bytes_type = DynSolType::Bytes;
        let bytes_json = json!("0x1234");
        let bytes_result = json_to_sol_value(&bytes_json, &bytes_type);
        assert!(bytes_result.is_ok());

        // Test fixed bytes conversion
        let fixed_bytes_type = DynSolType::FixedBytes(32);
        let fixed_bytes_json =
            json!("0x1234567890123456789012345678901234567890123456789012345678901234");
        let fixed_bytes_result = json_to_sol_value(&fixed_bytes_json, &fixed_bytes_type);
        assert!(fixed_bytes_result.is_ok());

        // Test invalid input type (e.g., number for address)
        let addr_invalid_json = json!(12345);
        let addr_invalid_result = json_to_sol_value(&addr_invalid_json, &addr_type);
        assert!(addr_invalid_result.is_err());
    }

    #[test]
    fn test_is_dynamic_type() {
        use alloy_dyn_abi::DynSolType;

        // Test dynamic types
        assert!(is_dynamic_type(&DynSolType::String));
        assert!(is_dynamic_type(&DynSolType::Bytes));
        assert!(is_dynamic_type(&DynSolType::Array(Box::new(DynSolType::Uint(256)))));
        assert!(is_dynamic_type(&DynSolType::Tuple(vec![
            DynSolType::Uint(256),
            DynSolType::Address
        ])));

        // Test static types
        assert!(!is_dynamic_type(&DynSolType::Address));
        assert!(!is_dynamic_type(&DynSolType::Uint(256)));
        assert!(!is_dynamic_type(&DynSolType::Bool));
        assert!(!is_dynamic_type(&DynSolType::FixedBytes(32)));
    }

    // Note: create_payload_from_tx and validate_transaction functions depend on Context
    // and would need more complex mocking for comprehensive testing
}
