use alloy_primitives::{Address, Bytes, U256};
use hex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tiny_keccak::{Hasher, Keccak};

/// Represents a smart contract that the DAO can interact with
#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub address: String,
    pub abi: String, // JSON ABI string
}

/// Helper methods for working with contracts
impl Contract {
    /// Create a new Contract instance
    pub fn new(name: &str, address: &str, abi: &str) -> Self {
        Self { name: name.to_string(), address: address.to_string(), abi: abi.to_string() }
    }

    /// Encode a function call for this contract using the ABI
    pub fn encode_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> Result<Bytes, String> {
        encode_function_call(&self.abi, function_name, args)
    }

    /// Find a function in the ABI
    pub fn find_function(&self, function_name: &str) -> Result<serde_json::Value, String> {
        // Parse the ABI
        let abi: serde_json::Value =
            serde_json::from_str(&self.abi).map_err(|e| format!("Failed to parse ABI: {}", e))?;

        // Get the array of functions from the ABI
        let functions = match abi {
            serde_json::Value::Array(funcs) => funcs,
            _ => return Err("ABI is not in expected format".to_string()),
        };

        // Find the specific function
        let function = functions
            .iter()
            .find(|func| {
                func.get("name").and_then(|n| n.as_str()) == Some(function_name)
                    && func.get("type").and_then(|t| t.as_str()) == Some("function")
            })
            .ok_or_else(|| format!("Function '{}' not found in ABI", function_name))?;

        Ok(function.clone())
    }

    /// Validate function arguments against the ABI
    pub fn validate_function_call(
        &self,
        function_name: &str,
        args: &[serde_json::Value],
    ) -> Result<(), String> {
        // Find the function in the ABI
        let function = self.find_function(function_name)?;

        // Get the inputs
        let inputs = function
            .get("inputs")
            .and_then(|i| i.as_array())
            .ok_or_else(|| format!("Function '{}' has no inputs defined", function_name))?;

        // Check argument count
        if inputs.len() != args.len() {
            return Err(format!(
                "Function '{}' expects {} arguments, but {} were provided",
                function_name,
                inputs.len(),
                args.len()
            ));
        }

        // Validate each argument
        for (i, (input, arg)) in inputs.iter().zip(args.iter()).enumerate() {
            let input_type = input
                .get("type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| format!("Input {} has no type", i))?;

            validate_argument(input_type, arg)
                .map_err(|e| format!("Invalid argument {}: {}", i, e))?;
        }

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

/// Dynamically encode a function call using the ABI
pub fn encode_function_call(
    abi_json: &str,
    function_name: &str,
    args: &[serde_json::Value],
) -> Result<Bytes, String> {
    // Parse the ABI to find the function
    let abi: serde_json::Value =
        serde_json::from_str(abi_json).map_err(|e| format!("Failed to parse ABI: {}", e))?;

    // Get the array of functions from the ABI
    let functions = match abi {
        serde_json::Value::Array(funcs) => funcs,
        _ => return Err("ABI is not in expected format".to_string()),
    };

    // Find the specific function
    let function = functions
        .iter()
        .find(|func| {
            func.get("name").and_then(|n| n.as_str()) == Some(function_name)
                && func.get("type").and_then(|t| t.as_str()) == Some("function")
        })
        .ok_or_else(|| format!("Function '{}' not found in ABI", function_name))?;

    // Get the inputs
    let inputs = function
        .get("inputs")
        .and_then(|i| i.as_array())
        .ok_or_else(|| format!("Function '{}' has no inputs defined", function_name))?;

    // Ensure we have the right number of arguments
    if inputs.len() != args.len() {
        return Err(format!(
            "Function '{}' expects {} arguments, but {} were provided",
            function_name,
            inputs.len(),
            args.len()
        ));
    }

    // For any contract function, use the generic ethabi approach
    // Create a function selector (first 4 bytes of the keccak256 hash of the function signature)
    let mut selector = [0u8; 4];

    // Build the function signature (e.g., "transfer(address,uint256)")
    let mut signature = function_name.to_string();
    signature.push('(');

    for (i, input) in inputs.iter().enumerate() {
        if let Some(type_str) = input.get("type").and_then(|t| t.as_str()) {
            if i > 0 {
                signature.push(',');
            }
            signature.push_str(type_str);
        }
    }

    signature.push(')');

    // Hash the signature
    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    hasher.finalize(&mut selector);

    // Start with the selector
    let mut calldata = selector.to_vec();

    // Encode each argument
    for (i, arg) in args.iter().enumerate() {
        if let Some(input_type) =
            inputs.get(i).and_then(|input| input.get("type")).and_then(|t| t.as_str())
        {
            encode_argument(&mut calldata, input_type, arg)?;
        }
    }

    Ok(Bytes::from(calldata))
}

/// Encode a single argument based on its Solidity type
/// This is a simplified implementation that handles common types
pub fn encode_argument(
    calldata: &mut Vec<u8>,
    type_str: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    match type_str {
        // Address type
        "address" => {
            let address_str = value.as_str().ok_or("Address must be a string")?;
            let address = Address::from_str(address_str)
                .map_err(|_| format!("Invalid address: {}", address_str))?;

            // Pad to 32 bytes
            let mut padded = [0u8; 32];
            padded[12..].copy_from_slice(address.as_slice());
            calldata.extend_from_slice(&padded);
        }

        // Uint types
        t if t.starts_with("uint") => {
            let num_str = value.as_str().ok_or("Number must be a string")?;
            let num =
                U256::from_str(num_str).map_err(|_| format!("Invalid number: {}", num_str))?;

            // Convert U256 to bytes
            let bytes = num.to_be_bytes::<32>();
            calldata.extend_from_slice(&bytes);
        }

        // Boolean type
        "bool" => {
            let bool_val = value.as_bool().ok_or("Boolean value expected")?;

            // Pad to 32 bytes
            let mut padded = [0u8; 32];
            if bool_val {
                padded[31] = 1;
            }
            calldata.extend_from_slice(&padded);
        }

        // String and bytes - dynamic types require more complex encoding
        "string" => {
            return Err(format!(
                "Dynamic type string not supported in this simplified implementation"
            ));
        }

        // Bytes type validation
        t if t.starts_with("bytes") => {
            if t == "bytes32" {
                // Fixed size bytes can be handled
                let bytes_str = value.as_str().ok_or("Bytes must be a hex string")?;
                if !bytes_str.starts_with("0x") {
                    return Err("Bytes must start with 0x".to_string());
                }

                // Decode hex
                let hex_str = &bytes_str[2..];
                if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err("Bytes contain invalid characters".to_string());
                }

                // Create padded bytes
                let mut padded = [0u8; 32];
                if hex_str.len() <= 64 {
                    // 32 bytes = 64 hex chars
                    let bytes =
                        hex::decode(hex_str).map_err(|_| "Invalid hex string".to_string())?;

                    // Copy to padded buffer, right-aligned
                    let start = 32 - bytes.len();
                    padded[start..].copy_from_slice(&bytes);
                } else {
                    return Err("bytes32 value too long".to_string());
                }

                calldata.extend_from_slice(&padded);
            } else {
                return Err(format!(
                    "Dynamic bytes type {} not supported in this implementation",
                    t
                ));
            }
        }

        // Unsupported type
        _ => {
            return Err(format!("Unsupported type: {}", type_str));
        }
    }

    Ok(())
}

/// Validate a single argument against its expected Solidity type
pub fn validate_argument(type_str: &str, value: &serde_json::Value) -> Result<(), String> {
    match type_str {
        // Address validation
        "address" => {
            let address_str = value.as_str().ok_or("Address must be a string")?;
            if address_str.len() != 42 || !address_str.starts_with("0x") {
                return Err("Invalid address format".to_string());
            }
            // Check if it's a valid hex string
            if !address_str[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Err("Address contains invalid characters".to_string());
            }
        }

        // Uint validation
        t if t.starts_with("uint") => {
            let num_str = value.as_str().ok_or("Number must be a string")?;
            if let Err(e) = U256::from_str(num_str) {
                return Err(format!("Invalid number: {}", e));
            }
        }

        // Bool validation
        "bool" => {
            if !value.is_boolean() {
                return Err("Expected a boolean value".to_string());
            }
        }

        // String validation
        "string" => {
            if !value.is_string() {
                return Err("Expected a string value".to_string());
            }
        }

        // Bytes validation
        t if t.starts_with("bytes") => {
            let bytes_str = value.as_str().ok_or("Bytes must be a hex string")?;
            if !bytes_str.starts_with("0x") {
                return Err("Bytes must start with 0x".to_string());
            }
            if !bytes_str[2..].chars().all(|c| c.is_ascii_hexdigit()) {
                return Err("Bytes contain invalid characters".to_string());
            }
        }

        // Unsupported type
        _ => {
            return Err(format!("Validation for type {} not implemented", type_str));
        }
    }

    Ok(())
}

/// Represents a token balance
#[derive(Debug, Clone)]
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
