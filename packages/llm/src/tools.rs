use crate::bindings::exports::wavs::agent::client::LlmClient;
use crate::bindings::exports::wavs::agent::tools::{self};
use crate::bindings::exports::wavs::agent::types::{
    Contract, CustomToolHandler, Function, Message, Tool, ToolCall,
};
use serde_json::{json, Value};

// Implementation for ToolsBuilder
pub struct ToolsBuilderImpl;

impl tools::GuestToolsBuilder for ToolsBuilderImpl {
    fn send_eth_tool(&self) -> Tool {
        Tool {
            tool_type: "function".into(),
            function: Function {
                name: "send_eth".into(),
                description: Some("Send ETH to an address".into()),
                parameters: Some(r#"{
                    "type": "object",
                    "properties": {
                        "to": {
                            "type": "string",
                            "description": "Ethereum address to send ETH to"
                        },
                        "value": {
                            "type": "string",
                            "description": "Amount of ETH to send in wei"
                        },
                        "data": {
                            "type": "string",
                            "description": "Hex-encoded transaction data, usually '0x' for simple transfers"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the transaction"
                        }
                    },
                    "required": ["to", "value"]
                }"#.into()),
            },
        }
    }

    fn tools_from_contract(&self, contract: Contract) -> Vec<Tool> {
        let mut tools = Vec::new();

        // Parse the ABI
        let abi_value: Result<serde_json::Value, _> = serde_json::from_str(&contract.abi);
        if abi_value.is_err() {
            println!("Failed to parse ABI: {:?}", abi_value.err());
            return tools;
        }

        let abi = abi_value.unwrap();

        // ABI can be either an array or an object with an "abi" field
        let functions = if abi.is_array() {
            abi.as_array().unwrap()
        } else if abi.is_object() && abi.get("abi").is_some() && abi["abi"].is_array() {
            abi["abi"].as_array().unwrap()
        } else {
            println!("Unexpected ABI format");
            return tools;
        };

        // Process each function in the ABI
        for func in functions {
            // Skip if not a function or is not externally callable
            // Handle both newer ABIs with stateMutability and older ABIs with constant field
            if !func.is_object()
                || func.get("type").is_none()
                || func["type"] != "function"
                || (func.get("stateMutability").is_none() && func.get("constant").is_none())
                || (func.get("stateMutability").is_some()
                    && func["stateMutability"] != "nonpayable"
                    && func["stateMutability"] != "payable")
                || (func.get("constant").is_some() && func["constant"] == true)
            {
                continue;
            }

            let name = match func.get("name") {
                Some(n) if n.is_string() => n.as_str().unwrap(),
                _ => continue, // Skip if no valid name
            };

            // Create properties for the function inputs
            let mut properties = json!({});
            let mut required = Vec::new();

            // Add value field for payable functions
            if func.get("stateMutability").map_or(false, |s| s == "payable") {
                properties["value"] = json!({
                    "type": "string",
                    "description": "Amount of ETH to send with the call (in wei)"
                });
                required.push("value");
            }

            // Process function inputs
            if let Some(inputs) = func.get("inputs").and_then(|i| i.as_array()) {
                for input in inputs {
                    if let (Some(param_name), Some(param_type)) = (
                        input.get("name").and_then(|n| n.as_str()),
                        input.get("type").and_then(|t| t.as_str()),
                    ) {
                        // Only skip empty param names
                        if param_name.is_empty() {
                            continue;
                        }

                        // Convert Solidity type to JSON Schema type
                        let (json_type, format) = solidity_type_to_json_schema(param_type);

                        let mut param_schema = json!({
                            "type": json_type,
                            "description": format!("{} ({})", param_name, param_type)
                        });

                        // Add format if specified
                        if let Some(fmt) = format {
                            param_schema["format"] = json!(fmt);
                        }

                        properties[param_name] = param_schema;
                        required.push(param_name);
                    }
                }
            }

            // Create the tool for this function
            let tool_name = format!("contract_{}_{}", contract.name.to_lowercase(), name);
            let tool = Tool {
                tool_type: "function".into(),
                function: Function {
                    name: tool_name.clone(),
                    description: Some(format!(
                        "Call the {} function on the {} contract at {}",
                        name, contract.name, contract.address
                    )),
                    parameters: Some(
                        json!({
                            "type": "object",
                            "properties": properties,
                            "required": required
                        })
                        .to_string(),
                    ),
                },
            };

            tools.push(tool);
        }

        tools
    }

    fn custom_tool(&self, name: String, description: String, parameters: String) -> Tool {
        Tool {
            tool_type: "function".into(),
            function: Function {
                name,
                description: Some(description),
                parameters: Some(parameters),
            },
        }
    }

    fn execute_tool_call(
        &self,
        tool_call: ToolCall,
        custom_handlers: Option<Vec<CustomToolHandler>>,
    ) -> Result<String, String> {
        let function_name = &tool_call.function.name;

        // If there are custom handlers, we'd need to manually check each one
        // But since CustomToolHandler is a resource type and doesn't have direct methods,
        // we can't actually implement this in the component version
        // So we'll just skip custom handlers for now
        if custom_handlers.is_some() {
            println!("Custom handlers provided, but not supported in this implementation");
        }

        // Use built-in handlers instead
        match function_name.as_str() {
            "send_eth" => self.parse_eth_transaction(tool_call),
            // Handle dynamically generated contract tools
            _ if function_name.starts_with("contract_") => parse_contract_function_call(&tool_call),
            _ => Err(format!("Unknown tool: {}", function_name)),
        }
    }

    fn parse_eth_transaction(&self, tool_call: ToolCall) -> Result<String, String> {
        // Parse the tool call arguments
        let args: Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse transaction arguments: {}", e))?;

        // Create a transaction JSON from the arguments with default values for optional fields
        let transaction = json!({
            "to": args["to"].as_str().ok_or("Missing 'to' field")?,
            "value": args["value"].as_str().ok_or("Missing 'value' field")?,
            "data": args["data"].as_str().unwrap_or("0x"),
            "description": args["description"].as_str().unwrap_or("ETH transfer"),
            "contract_call": null
        });

        // Serialize back to a string for passing between functions
        let tx_json = serde_json::to_string(&transaction)
            .map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(tx_json)
    }

    fn process_tool_calls(
        &self,
        _client: LlmClient,
        _initial_messages: Vec<Message>,
        _response: Message,
        tool_calls: Vec<ToolCall>,
        _custom_handlers: Option<Vec<CustomToolHandler>>,
    ) -> Result<String, String> {
        // Process each tool call and collect the results
        let mut tool_results = Vec::new();
        for tool_call in &tool_calls {
            // We can't pass custom_handlers because it can't be cloned
            let tool_result = self.execute_tool_call(tool_call.clone(), None)?;
            tool_results.push(tool_result);
        }

        // For simplicity, we'll just return the first tool result or a combined result
        if tool_results.len() == 1 {
            Ok(tool_results[0].clone())
        } else {
            // For multiple tool calls, combine the results
            Ok(tool_results.join("\n"))
        }
    }
}

/// Convert Solidity type to JSON Schema type
fn solidity_type_to_json_schema(solidity_type: &str) -> (&'static str, Option<&'static str>) {
    match solidity_type {
        t if t.starts_with("uint") => ("string", None), // Use string for all integers to handle large numbers
        t if t.starts_with("int") => ("string", None),
        "address" => ("string", Some("ethereum-address")),
        "bool" => ("boolean", None),
        "string" => ("string", None),
        t if t.starts_with("bytes") => ("string", Some("byte")),
        _ => ("string", None), // Default to string for unknown types
    }
}

// Helper function to parse contract function calls
fn parse_contract_function_call(tool_call: &ToolCall) -> Result<String, String> {
    // Extract contract name and function from the tool name
    // Format is "contract_{contract_name}_{function_name}"
    let parts: Vec<&str> = tool_call.function.name.splitn(3, '_').collect();
    if parts.len() < 3 {
        return Err(format!("Invalid contract tool name: {}", tool_call.function.name));
    }

    let contract_name = parts[1];
    let function_name = parts[2];

    // Parse the arguments
    let args: Value = serde_json::from_str(&tool_call.function.arguments)
        .map_err(|e| format!("Failed to parse function arguments: {}", e))?;

    // Create contract call
    let mut function_args = Vec::new();
    let mut value = "0".to_string();

    // Collect all args, and handle 'value' for ETH transfers specially
    if let Some(obj) = args.as_object() {
        for (key, val) in obj {
            if key == "value" {
                value = val.as_str().unwrap_or("0").to_string();
            } else {
                function_args.push(val.clone());
            }
        }
    }

    // Create a transaction JSON
    let transaction = json!({
        "to": format!("contract_addr_{}", contract_name), // Placeholder
        "value": value,
        "data": "0x", // Will be encoded by the execution layer
        "description": format!("Calling {} on {} contract", function_name, contract_name),
        "contract_call": {
            "function": function_name,
            "args": function_args
        }
    });

    // Serialize to JSON
    let tx_json = serde_json::to_string(&transaction)
        .map_err(|e| format!("Failed to serialize transaction: {}", e))?;

    Ok(tx_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::exports::wavs::agent::tools::GuestToolsBuilder;
    use crate::bindings::exports::wavs::agent::types::ToolCallFunction;

    // Helper function to create a test ToolCall
    fn create_test_tool_call(id: &str, name: &str, arguments: &str) -> ToolCall {
        ToolCall {
            id: id.into(),
            tool_type: "function".into(),
            function: ToolCallFunction { name: name.into(), arguments: arguments.into() },
        }
    }

    #[test]
    fn test_tools_builder() {
        let builder = ToolsBuilderImpl;

        // Test send_eth_tool
        let eth_tool = builder.send_eth_tool();
        assert_eq!(eth_tool.tool_type, "function");
        assert_eq!(eth_tool.function.name, "send_eth");
        assert!(eth_tool.function.description.is_some());
        assert!(eth_tool.function.parameters.is_some());

        // Test custom_tool
        let custom_tool = builder.custom_tool(
            "test-tool".into(),
            "Test description".into(),
            r#"{"type":"object"}"#.into(),
        );
        assert_eq!(custom_tool.tool_type, "function");
        assert_eq!(custom_tool.function.name, "test-tool");
        assert_eq!(custom_tool.function.description, Some("Test description".into()));
        assert_eq!(custom_tool.function.parameters, Some(r#"{"type":"object"}"#.into()));
    }

    #[test]
    fn test_tools_from_contract() {
        let builder = ToolsBuilderImpl;

        // Create a sample contract
        let contract = Contract {
            name: "TestToken".into(),
            address: "0x1234567890123456789012345678901234567890".into(),
            abi: r#"[
                {
                    "type": "function",
                    "name": "transfer",
                    "stateMutability": "nonpayable",
                    "inputs": [
                        {
                            "name": "to",
                            "type": "address"
                        },
                        {
                            "name": "value",
                            "type": "uint256"
                        }
                    ]
                }
            ]"#
            .into(),
            description: None,
        };

        let tools = builder.tools_from_contract(contract);

        // Verify the tool was created correctly
        assert!(!tools.is_empty());
        let tool = &tools[0];
        assert_eq!(tool.tool_type, "function");
        assert!(tool.function.name.contains("contract_testtoken_transfer"));
        assert!(tool.function.description.is_some());
        assert!(tool.function.parameters.is_some());

        // Check the parameters schema
        if let Some(params) = &tool.function.parameters {
            let schema: Value = serde_json::from_str(params).unwrap();
            assert!(schema["properties"].is_object());
            assert!(schema["required"].is_array());
        }
    }

    #[test]
    fn test_parse_eth_transaction() {
        let builder = ToolsBuilderImpl;

        // Create a valid tool call
        let tool_call = create_test_tool_call(
            "123",
            "send_eth",
            r#"{
                "to": "0x1234567890123456789012345678901234567890",
                "value": "1000000000000000000",
                "description": "Test transaction"
            }"#,
        );

        let result = builder.parse_eth_transaction(tool_call);
        assert!(result.is_ok());

        // Parse the result to check the fields
        let tx_json: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(tx_json["to"], "0x1234567890123456789012345678901234567890");
        assert_eq!(tx_json["value"], "1000000000000000000");
        assert_eq!(tx_json["description"], "Test transaction");
        assert_eq!(tx_json["data"], "0x");
        assert!(tx_json["contract_call"].is_null());

        // Test with missing fields
        let invalid_tool_call = create_test_tool_call(
            "456",
            "send_eth",
            r#"{"description": "Missing required fields"}"#,
        );

        let invalid_result = builder.parse_eth_transaction(invalid_tool_call);
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_execute_tool_call() {
        let builder = ToolsBuilderImpl;

        // Test with send_eth
        let eth_tool_call = create_test_tool_call(
            "123",
            "send_eth",
            r#"{
                "to": "0x1234567890123456789012345678901234567890",
                "value": "1000000000000000000"
            }"#,
        );

        let result = builder.execute_tool_call(eth_tool_call, None);
        assert!(result.is_ok());

        // Test with unknown tool
        let unknown_tool_call = create_test_tool_call("456", "unknown_tool", r#"{}"#);

        let unknown_result = builder.execute_tool_call(unknown_tool_call, None);
        assert!(unknown_result.is_err());
    }

    #[test]
    fn test_solidity_type_to_json_schema() {
        // Test address type
        let (addr_type, addr_format) = solidity_type_to_json_schema("address");
        assert_eq!(addr_type, "string");
        assert_eq!(addr_format, Some("ethereum-address"));

        // Test uint type
        let (uint_type, uint_format) = solidity_type_to_json_schema("uint256");
        assert_eq!(uint_type, "string");
        assert_eq!(uint_format, None);

        // Test bool type
        let (bool_type, bool_format) = solidity_type_to_json_schema("bool");
        assert_eq!(bool_type, "boolean");
        assert_eq!(bool_format, None);

        // Test string type
        let (string_type, string_format) = solidity_type_to_json_schema("string");
        assert_eq!(string_type, "string");
        assert_eq!(string_format, None);

        // Test bytes type
        let (bytes_type, bytes_format) = solidity_type_to_json_schema("bytes");
        assert_eq!(bytes_type, "string");
        assert_eq!(bytes_format, Some("byte"));
    }

    #[test]
    fn test_parse_contract_function_call() {
        // Test valid contract function call
        let tool_call = create_test_tool_call(
            "123",
            "contract_testtoken_transfer",
            r#"{
                "to": "0x1234567890123456789012345678901234567890",
                "value": "1000"
            }"#,
        );

        let result = parse_contract_function_call(&tool_call);
        assert!(result.is_ok());

        // Verify the transaction JSON
        let tx_json: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(tx_json["to"].as_str().unwrap().contains("contract_addr_testtoken"));
        assert_eq!(tx_json["value"], "1000");
        assert!(tx_json["description"].as_str().unwrap().contains("transfer"));
        assert_eq!(tx_json["contract_call"]["function"], "transfer");

        // Test with invalid tool name format
        // The function needs at least 3 parts when split by '_'
        let invalid_tool_call = create_test_tool_call("456", "invalid", r#"{}"#);

        let invalid_result = parse_contract_function_call(&invalid_tool_call);
        assert!(invalid_result.is_err());
    }
}
