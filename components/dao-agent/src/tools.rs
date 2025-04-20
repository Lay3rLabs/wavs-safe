use crate::llm::LLMClient;
use crate::models::SafeTransaction;
use serde::{Deserialize, Serialize};

/// Function parameter for tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub parameter_type: Option<String>,
}

/// Function definition for tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool definition for chat completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function,
}

/// Tool call for chat completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    #[serde(default = "default_tool_id")]
    pub id: String,
    #[serde(rename = "type")]
    #[serde(default = "default_tool_type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_arguments")]
    pub arguments: String,
}

/// Custom deserializer for function arguments that can be either a string or an object
fn deserialize_arguments<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    // First try to deserialize as a Value to handle both string and object
    let value = Value::deserialize(deserializer)?;

    match value {
        // If it's already a string, return it directly
        Value::String(s) => Ok(s),

        // If it's an object, convert it to a JSON string
        Value::Object(_) => serde_json::to_string(&value)
            .map_err(|e| D::Error::custom(format!("Failed to serialize object to string: {}", e))),

        // For any other type, try to convert to string representation
        _ => serde_json::to_string(&value)
            .map_err(|e| D::Error::custom(format!("Failed to serialize value to string: {}", e))),
    }
}

/// Common message structure for chat completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Tool result message
impl Message {
    pub fn new_user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn new_system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    pub fn new_tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
            name: None,
        }
    }
}

/// Helper functions to create common tools
pub mod builders {
    use super::*;
    use serde_json::json;

    /// Create a tool to send ETH through the DAO's Safe
    pub fn send_eth() -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "send_eth".to_string(),
                description: Some("Send ETH through the DAO's Gnosis Safe".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "to": {
                            "type": "string",
                            "description": "Destination address (0x...)"
                        },
                        "value": {
                            "type": "string",
                            "description": "Amount in wei to send (as string)"
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
                    "required": ["to", "value", "data", "description"]
                })),
            },
        }
    }

    /// Create a tool to send ERC20 tokens through the DAO's Safe
    pub fn send_erc20() -> Tool {
        Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "send_erc20".to_string(),
                description: Some(
                    "Send ERC20 tokens (like USDC) through the DAO's Gnosis Safe".to_string(),
                ),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "token_address": {
                            "type": "string",
                            "description": "Address of the ERC20 token contract (0x...)"
                        },
                        "token_symbol": {
                            "type": "string",
                            "description": "Symbol of the token (e.g., USDC)"
                        },
                        "to": {
                            "type": "string",
                            "description": "Recipient address (0x...)"
                        },
                        "amount": {
                            "type": "string",
                            "description": "Amount to send as a string (in token's smallest unit)"
                        },
                        "description": {
                            "type": "string",
                            "description": "Description of the transaction"
                        }
                    },
                    "required": ["token_address", "token_symbol", "to", "amount", "description"]
                })),
            },
        }
    }
}

/// Tool execution handlers
pub mod handlers {
    use super::*;
    use serde_json::Value;

    /// Execute a tool call and return the result
    pub fn execute_tool_call(tool_call: &ToolCall) -> Result<String, String> {
        match tool_call.function.name.as_str() {
            "send_eth" => parse_eth_transaction(tool_call),
            "send_erc20" => parse_erc20_transaction(tool_call),
            _ => Ok(format!("Unknown tool: {}", tool_call.function.name)),
        }
    }

    /// Parse an ETH transaction from tool call
    pub fn parse_eth_transaction(tool_call: &ToolCall) -> Result<String, String> {
        // Parse the tool call arguments
        let args: Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse transaction arguments: {}", e))?;

        // Create a SafeTransaction from the arguments
        let transaction = SafeTransaction {
            to: args["to"].as_str().ok_or("Missing 'to' field")?.to_string(),
            value: args["value"].as_str().ok_or("Missing 'value' field")?.to_string(),
            data: args["data"].as_str().ok_or("Missing 'data' field")?.to_string(),
            description: args["description"]
                .as_str()
                .ok_or("Missing 'description' field")?
                .to_string(),
            contract_call: None,
        };

        // Serialize back to a string for passing between functions
        let tx_json = serde_json::to_string(&transaction)
            .map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(tx_json)
    }

    /// Parse an ERC20 transaction from tool call
    pub fn parse_erc20_transaction(tool_call: &ToolCall) -> Result<String, String> {
        // Parse the tool call arguments
        let args: Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse transaction arguments: {}", e))?;

        // Extract required fields
        let token_address =
            args["token_address"].as_str().ok_or("Missing 'token_address' field")?.to_string();
        let to = args["to"].as_str().ok_or("Missing 'to' field")?.to_string();
        let amount = args["amount"].as_str().ok_or("Missing 'amount' field")?.to_string();
        let description =
            args["description"].as_str().ok_or("Missing 'description' field")?.to_string();

        // Create a contract call for the ERC20 transfer
        let contract_call = Some(crate::models::ContractCall {
            function: "transfer".to_string(),
            args: vec![serde_json::to_value(&to).unwrap(), serde_json::to_value(&amount).unwrap()],
        });

        // Create a SafeTransaction targeting the token contract
        let transaction = SafeTransaction {
            to: token_address,
            value: "0".to_string(), // No ETH is sent for ERC20 transfers
            data: "0x".to_string(), // Will be encoded by the execution layer
            description,
            contract_call,
        };

        // Serialize back to a string for passing between functions
        let tx_json = serde_json::to_string(&transaction)
            .map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(tx_json)
    }
}

/// Default function for tool ID
fn default_tool_id() -> String {
    // Generate a simple random ID when none is provided (e.g., by Ollama)
    format!(
        "call_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

/// Default function for tool type
fn default_tool_type() -> String {
    "function".to_string()
}

/// Process tool calls and generate a response
pub async fn process_tool_calls(
    client: &LLMClient,
    initial_messages: Vec<Message>,
    response: Message,
    tool_calls: Vec<ToolCall>,
) -> Result<String, String> {
    println!("Processing tool calls...");

    // Check if we're using Ollama based on the model name
    let model = client.get_model();
    // TODO: This is a hack and could be improved
    let is_ollama =
        model.starts_with("llama") || model.starts_with("mistral") || !model.contains("gpt");

    // Process each tool call and collect the results
    let mut tool_results = Vec::new();
    for tool_call in &tool_calls {
        let tool_result = handlers::execute_tool_call(tool_call)?;
        println!("Tool result: {}", tool_result);
        tool_results.push(tool_result);
    }

    if is_ollama {
        // For Ollama: Don't make a second call, just use the tool result directly
        println!("Using direct tool result handling for Ollama");

        if tool_results.len() == 1 {
            Ok(tool_results[0].clone())
        } else {
            // For multiple tool calls, combine the results
            Ok(tool_results.join("\n"))
        }
    } else {
        // For OpenAI: Use the standard tool calls protocol
        println!("Using OpenAI-compatible tool call handling");
        let mut tool_messages = initial_messages.clone();

        // Add the assistant's response with tool calls, ensuring content is not null
        // When we're sending tool calls, OpenAI requires content to be a string (even if empty)
        // We MUST preserve the original tool_calls so OpenAI can match the tool responses
        let sanitized_response = Message {
            role: response.role,
            content: Some(response.content.unwrap_or_default()),
            tool_calls: Some(tool_calls.clone()), // Important: preserve the tool_calls!
            tool_call_id: response.tool_call_id,
            name: response.name,
        };
        tool_messages.push(sanitized_response);

        // Process each tool call and add the results
        for (i, tool_call) in tool_calls.iter().enumerate() {
            tool_messages
                .push(Message::new_tool_result(tool_call.id.clone(), tool_results[i].clone()));
        }

        // Call OpenAI to get final response, but we don't use it for parsing
        // It's mainly for human readable confirmation
        let final_response = client.chat_completion_text(&tool_messages).await;
        println!("OpenAI final response (for logs only): {:?}", final_response);

        // Return the original tool result which contains valid JSON
        // Only handle the first tool result for now since we expect a single transaction
        if tool_results.len() >= 1 {
            Ok(tool_results[0].clone())
        } else {
            Err("No tool results available".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition() {
        // Define tools
        let eth_tool = builders::send_eth();
        let erc20_tool = builders::send_erc20();

        // Convert to JSON
        let eth_json = serde_json::to_string(&eth_tool).unwrap();
        let erc20_json = serde_json::to_string(&erc20_tool).unwrap();

        // Ensure they can be serialized and deserialized correctly
        let deserialized_eth: Tool = serde_json::from_str(&eth_json).unwrap();
        assert_eq!(deserialized_eth.tool_type, "function");
        assert_eq!(deserialized_eth.function.name, "send_eth");

        let deserialized_erc20: Tool = serde_json::from_str(&erc20_json).unwrap();
        assert_eq!(deserialized_erc20.tool_type, "function");
        assert_eq!(deserialized_erc20.function.name, "send_erc20");
    }
}
