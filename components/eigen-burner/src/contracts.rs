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
    fn parse_abi(&self) -> Result<JsonAbi, String> {
        serde_json::from_str(&self.abi).map_err(|e| format!("Failed to parse ABI: {}", e))
    }

    /// Encode a function call for this contract using the ABI
    pub fn encode_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> Result<Bytes, String> {
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
    pub fn find_function(&self, function_name: &str) -> Result<Function, String> {
        let json_abi = self.parse_abi()?;

        json_abi
            .functions()
            .find(|f| f.name == function_name)
            .cloned()
            .ok_or_else(|| format!("Function '{}' not found in ABI", function_name))
    }

    /// Validate function arguments against the ABI
    pub fn validate_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> Result<(), String> {
        // Find the function in the ABI
        let function = self.find_function(function_name)?;

        // Check argument count
        if function.inputs.len() != args.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, but {} were provided",
                function_name,
                function.inputs.len(),
                args.len()
            ));
        }

        // Try encoding the arguments - if it fails, it's invalid
        encode_function_args(&function, args)?;

        Ok(())
    }
}

/// Represents a contract function call
#[derive(Serialize, Deserialize, Debug)]
pub struct ContractCall {
    pub function: String,
    pub args: Vec<serde_json::Value>,
}

/// Default function for ContractCall
pub fn default_contract_call() -> Option<ContractCall> {
    None
}

/// Convert a string to a DynSolValue based on the type
fn json_to_sol_value(value: &serde_json::Value, ty: &DynSolType) -> Result<DynSolValue, String> {
    match ty {
        DynSolType::Address => {
            // Convert string address to DynSolValue::Address
            let addr_str = value.as_str().ok_or("Address must be a string")?;
            let address = Address::from_str(addr_str)
                .map_err(|_| format!("Invalid address: {}", addr_str))?;
            Ok(DynSolValue::Address(address))
        }
        DynSolType::Uint(bits) => {
            // Convert string number to DynSolValue::Uint
            let num_str = value.as_str().ok_or("Number must be a string")?;
            let num =
                U256::from_str(num_str).map_err(|_| format!("Invalid number: {}", num_str))?;
            Ok(DynSolValue::Uint(num, *bits))
        }
        DynSolType::Bool => {
            // Convert JSON boolean to DynSolValue::Bool
            let bool_val = value.as_bool().ok_or("Expected a boolean value")?;
            Ok(DynSolValue::Bool(bool_val))
        }
        DynSolType::String => {
            // Convert JSON string to DynSolValue::String
            let string_val = value.as_str().ok_or("Expected a string value")?;
            Ok(DynSolValue::String(string_val.to_string()))
        }
        DynSolType::Bytes => {
            // Convert hex string to DynSolValue::Bytes
            let bytes_str = value.as_str().ok_or("Bytes must be a hex string")?;
            if !bytes_str.starts_with("0x") {
                return Err("Bytes must start with 0x".to_string());
            }
            let hex_str = &bytes_str[2..];
            let bytes = hex::decode(hex_str).map_err(|_| "Invalid hex string".to_string())?;
            Ok(DynSolValue::Bytes(bytes))
        }
        DynSolType::FixedBytes(size) => {
            // Convert hex string to fixed-size bytes
            let bytes_str = value.as_str().ok_or("Bytes must be a hex string")?;
            if !bytes_str.starts_with("0x") {
                return Err("Bytes must start with 0x".to_string());
            }
            let hex_str = &bytes_str[2..];
            let bytes = hex::decode(hex_str).map_err(|_| "Invalid hex string".to_string())?;

            if bytes.len() > *size {
                return Err(format!("Hex string too long for bytes{}", size));
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
        _ => Err(format!("Unsupported type: {:?}", ty)),
    }
}

/// Encode function arguments using Alloy's built-in functionality
fn encode_function_args(
    function: &Function,
    args: &[serde_json::Value],
) -> Result<Vec<u8>, String> {
    // If there are no arguments, return an empty vector
    if args.is_empty() {
        return Ok(Vec::new());
    }

    // Parse each parameter's type
    let param_types: Vec<DynSolType> = function
        .inputs
        .iter()
        .map(|param| {
            DynSolType::parse(&param.ty)
                .map_err(|e| format!("Invalid parameter type '{}': {}", param.ty, e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Convert each JSON value to a DynSolValue
    let mut values = Vec::with_capacity(args.len());

    for (i, (arg, ty)) in args.iter().zip(&param_types).enumerate() {
        match json_to_sol_value(arg, ty) {
            Ok(value) => values.push(value),
            Err(e) => return Err(format!("Error converting argument {}: {}", i, e)),
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

/// Check if a type is dynamic (string, bytes, arrays)
fn is_dynamic_type(ty: &DynSolType) -> bool {
    match ty {
        DynSolType::String => true,
        DynSolType::Bytes => true,
        DynSolType::Array(_) => true,
        DynSolType::FixedArray(_, size) => *size == 0,
        _ => false,
    }
}

/// Represents a supported token in the DAO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedToken {
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    pub description: Option<String>,
}

impl SupportedToken {
    /// Create a new SupportedToken instance
    pub fn new(address: &str, symbol: &str, decimals: u8) -> Self {
        Self {
            address: address.to_string(),
            symbol: symbol.to_string(),
            decimals,
            description: None,
        }
    }

    /// Create a new SupportedToken instance with description
    pub fn new_with_description(
        address: &str,
        symbol: &str,
        decimals: u8,
        description: &str,
    ) -> Self {
        Self {
            address: address.to_string(),
            symbol: symbol.to_string(),
            decimals,
            description: Some(description.to_string()),
        }
    }
}

/// Represents a token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_address: String,
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
}

/// Helper methods for token balances
impl TokenBalance {
    /// Create a new TokenBalance instance
    pub fn new(token_address: &str, symbol: &str, balance: &str, decimals: u8) -> Self {
        Self {
            token_address: token_address.to_string(),
            symbol: symbol.to_string(),
            balance: balance.to_string(),
            decimals,
        }
    }

    /// Format the balance for display with proper decimal places
    pub fn formatted_balance(&self) -> String {
        let raw_balance = self.balance.parse::<u128>().unwrap_or(0);
        let divisor = 10u128.pow(self.decimals as u32);

        if divisor == 0 {
            return format!("{}", raw_balance);
        }

        let whole_part = raw_balance / divisor;
        let decimal_part = raw_balance % divisor;

        // Format with proper decimal places
        if decimal_part == 0 {
            format!("{}", whole_part)
        } else {
            // Pad decimal part with leading zeros if needed
            let decimal_str = format!("{:0width$}", decimal_part, width = self.decimals as usize);
            // Trim trailing zeros
            let trimmed = decimal_str.trim_end_matches('0');
            if trimmed.is_empty() {
                format!("{}", whole_part)
            } else {
                format!("{}.{}", whole_part, trimmed)
            }
        }
    }
}
