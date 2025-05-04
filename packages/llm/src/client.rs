use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::env;
use wstd::{
    http::{Client, HeaderValue, IntoBody, Request, StatusCode},
    io::AsyncRead,
    runtime::block_on,
};

use crate::bindings::exports::wavs::agent::client;
use crate::bindings::exports::wavs::agent::errors::AgentError;
use crate::bindings::exports::wavs::agent::tools::GuestToolsBuilder;
use crate::bindings::exports::wavs::agent::types::{
    Config, CustomToolHandler, LlmOptions, LlmResponse, Message, Tool,
};

// JSON serializable version of LlmOptions
#[derive(Serialize, Deserialize)]
struct LlmOptionsJson {
    temperature: f32,
    top_p: f32,
    seed: u32,
    max_tokens: Option<u32>,
    context_window: Option<u32>,
}

impl From<LlmOptions> for LlmOptionsJson {
    fn from(options: LlmOptions) -> Self {
        Self {
            temperature: options.temperature,
            top_p: options.top_p,
            seed: options.seed,
            max_tokens: options.max_tokens,
            context_window: options.context_window,
        }
    }
}

impl From<LlmOptionsJson> for LlmOptions {
    fn from(json: LlmOptionsJson) -> Self {
        Self {
            temperature: json.temperature,
            top_p: json.top_p,
            seed: json.seed,
            max_tokens: json.max_tokens,
            context_window: json.context_window,
        }
    }
}

pub struct LlmClientImpl {
    pub model: String,
    pub config: LlmOptions,
    pub api_url: String,
    pub api_key: Option<String>,
}

impl client::GuestLlmClient for LlmClientImpl {
    fn new(&self, model: String) -> Result<client::LlmClient, AgentError> {
        // Validate model name
        if model.trim().is_empty() {
            return Err(AgentError::Llm("Model name cannot be empty".into()));
        }

        // Get API key if using OpenAI models
        let api_key = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                match std::env::var("WAVS_ENV_OPENAI_API_KEY") {
                    Ok(key) => Some(key),
                    Err(_) => None, // Only read if exists, don't return an error
                }
            }
            _ => None, // Local models don't need an API key
        };

        // Set API URL based on model type
        let api_url = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                "https://api.openai.com/v1/chat/completions".to_string()
            }
            _ => format!(
                "{}/api/chat",
                env::var("WAVS_ENV_OLLAMA_API_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string())
            ),
        };

        // Default configuration
        let config = LlmOptions {
            temperature: 0.7,
            top_p: 1.0,
            seed: 0,
            max_tokens: None,
            context_window: None,
        };

        Ok(client::LlmClient::new(Self { model, config, api_url, api_key }))
    }

    fn from_json(
        &self,
        model: String,
        json_config: String,
    ) -> Result<client::LlmClient, AgentError> {
        let config_json: LlmOptionsJson = serde_json::from_str(&json_config)
            .map_err(|e| AgentError::Other(format!("Invalid JSON: {}", e)))?;

        // Create a new LlmClient with the provided model and JSON config
        let mut client = LlmClientImpl {
            model: model.clone(),
            config: config_json.into(),
            api_url: String::new(),
            api_key: None,
        };

        // Get API key if using OpenAI models
        let api_key = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                match std::env::var("WAVS_ENV_OPENAI_API_KEY") {
                    Ok(key) => Some(key),
                    Err(_) => None,
                }
            }
            _ => None,
        };

        // Set API URL based on model type
        let api_url = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                "https://api.openai.com/v1/chat/completions".to_string()
            }
            _ => format!(
                "{}/api/chat",
                env::var("WAVS_ENV_OLLAMA_API_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string())
            ),
        };

        // Update client with API key and URL
        client.api_key = api_key;
        client.api_url = api_url;

        Ok(client::LlmClient::new(client))
    }

    fn with_config(
        &self,
        model: String,
        config: LlmOptions,
    ) -> Result<client::LlmClient, AgentError> {
        // Create a new LlmClient with the provided model and config
        let mut client =
            LlmClientImpl { model: model.clone(), config, api_url: String::new(), api_key: None };

        // Get API key if using OpenAI models
        let api_key = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                match std::env::var("WAVS_ENV_OPENAI_API_KEY") {
                    Ok(key) => Some(key),
                    Err(_) => None,
                }
            }
            _ => None,
        };

        // Set API URL based on model type
        let api_url = match model.as_str() {
            "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo" => {
                "https://api.openai.com/v1/chat/completions".to_string()
            }
            _ => format!(
                "{}/api/chat",
                env::var("WAVS_ENV_OLLAMA_API_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string())
            ),
        };

        // Update client with API key and URL
        client.api_key = api_key;
        client.api_url = api_url;

        Ok(client::LlmClient::new(client))
    }

    fn get_model(&self) -> String {
        self.model.clone()
    }

    fn get_config(&self) -> LlmOptions {
        self.config.clone()
    }

    fn chat_completion(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<Message, AgentError> {
        block_on(async {
            // Validate messages
            if messages.is_empty() {
                return Err(AgentError::Llm("Messages cannot be empty".into()));
            }

            println!("Sending chat completion request:");
            println!("- Model: {}", self.model);
            println!("- Number of messages: {}", messages.len());
            println!("- Tools provided: {}", tools.is_some());
            println!("- Temperature: {}", self.config.temperature);
            println!("- Top_p: {}", self.config.top_p);

            // Check if OpenAI models have an API key
            let is_openai_model = matches!(
                self.model.as_str(),
                "gpt-3.5-turbo" | "gpt-4" | "gpt-4o" | "gpt-4o-mini" | "gpt-4.1" | "gpt-4-turbo"
            );

            if is_openai_model && self.api_key.is_none() {
                return Err(AgentError::Llm("OpenAI API key is required for OpenAI models".into()));
            }

            // Calculate max tokens based on tools presence if not explicitly set
            let max_tokens =
                self.config.max_tokens.unwrap_or_else(|| if tools.is_some() { 1024 } else { 100 });

            // Create request body with configurable settings
            let body = if self.api_key.is_some() {
                // OpenAI format
                let mut request = serde_json::json!({
                    "model": self.model,
                    "messages": messages,
                    "temperature": self.config.temperature,
                    "top_p": self.config.top_p,
                    "seed": self.config.seed,
                    "stream": false,
                    "max_tokens": max_tokens
                });

                // Add tools if provided
                if let Some(tools_list) = tools {
                    request["tools"] = serde_json::to_value(tools_list)?;
                }

                request
            } else {
                // Ollama chat format
                let mut request = serde_json::json!({
                    "model": self.model,
                    "messages": messages,
                    "stream": false,
                    "options": {
                        "temperature": self.config.temperature,
                        "top_p": self.config.top_p,
                        "seed": self.config.seed,
                        "num_predict": max_tokens,
                    }
                });

                // Add context window if specified
                if let Some(ctx) = self.config.context_window {
                    request["options"]["num_ctx"] = serde_json::json!(ctx);
                }

                // Add tools if provided for Ollama (using the format Ollama expects)
                if let Some(tools_list) = tools.clone() {
                    // Standard tools format
                    request["tools"] = serde_json::to_value(tools_list.clone())?;

                    // Also include functions key which some Ollama versions might need
                    // Convert tools to format compatible with Ollama
                    let functions = tools_list
                        .iter()
                        .map(|tool| {
                            serde_json::json!({
                                "name": tool.function.name,
                                "description": tool.function.description,
                                "parameters": tool.function.parameters
                            })
                        })
                        .collect::<Vec<_>>();

                    request["functions"] = serde_json::json!(functions);
                }

                request
            };

            println!("Request body: {}", serde_json::to_string_pretty(&body).unwrap_or_default());

            // Create request
            let mut req = Request::post(&self.api_url)
                .body(serde_json::to_vec(&body).unwrap().into_body())
                .map_err(|e| AgentError::Http(format!("Failed to create request: {}", e)))?;

            // Add headers
            req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
            req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

            // Add authorization if needed
            if let Some(api_key) = &self.api_key {
                req.headers_mut().insert(
                    "Authorization",
                    HeaderValue::from_str(&format!("Bearer {}", api_key))
                        .map_err(|e| AgentError::Http(format!("Invalid API key format: {}", e)))?,
                );
            }

            println!("Sending request to: {}", req.uri());

            // Send request
            let mut res = Client::new()
                .send(req)
                .await
                .map_err(|e| AgentError::Http(format!("Request failed: {}", e)))?;

            println!("Received response with status: {}", res.status());

            if res.status() != StatusCode::OK {
                let mut error_body = Vec::new();
                res.body_mut()
                    .read_to_end(&mut error_body)
                    .await
                    .map_err(|e| AgentError::Io(format!("Failed to read error response: {}", e)))?;
                let error_msg = format!(
                    "API error: status {} - {}",
                    res.status(),
                    String::from_utf8_lossy(&error_body)
                );
                println!("Error: {}", error_msg);
                return Err(AgentError::Llm(error_msg));
            }

            // Read response body
            let mut body_buf = Vec::new();
            res.body_mut()
                .read_to_end(&mut body_buf)
                .await
                .map_err(|e| AgentError::Io(format!("Failed to read response body: {}", e)))?;

            let body_str = String::from_utf8(body_buf)
                .map_err(|e| AgentError::Utf8(format!("Invalid UTF-8 in response: {}", e)))?;

            println!("Raw response: {}", body_str);

            // Parse response based on provider
            if self.api_key.is_some() {
                // Parse OpenAI response format
                #[derive(Deserialize)]
                struct ChatResponse {
                    choices: Vec<Choice>,
                }

                #[derive(Deserialize)]
                struct Choice {
                    message: Message,
                }

                let resp: ChatResponse = serde_json::from_str(&body_str).map_err(|e| {
                    AgentError::Llm(format!("Failed to parse OpenAI response: {}", e))
                })?;

                resp.choices
                    .first()
                    .map(|choice| choice.message.clone())
                    .ok_or_else(|| AgentError::Llm("No response choices returned".into()))
            } else {
                // Parse Ollama chat response format
                #[derive(Debug, Deserialize)]
                struct OllamaResponse {
                    message: Message,
                    #[serde(default)]
                    _model: String,
                    #[serde(default)]
                    _created_at: String,
                }

                // Parse as Ollama response
                let resp: OllamaResponse = serde_json::from_str(&body_str).map_err(|e| {
                    AgentError::Llm(format!("Failed to parse Ollama response: {}", e))
                })?;

                Ok(resp.message)
            }
        })
    }

    fn chat_completion_text(&self, messages: Vec<Message>) -> Result<String, AgentError> {
        let response = self.chat_completion(messages, None)?;
        Ok(response.content.unwrap_or_default())
    }

    fn process_prompt(
        &self,
        prompt: String,
        config: Config,
        custom_tools: Option<Vec<Tool>>,
        custom_handlers: Option<Vec<CustomToolHandler>>,
    ) -> Result<LlmResponse, AgentError> {
        // Create the tools for ETH transfers
        let eth_tool = create_send_eth_tool();

        // Generate tools from smart contract ABIs
        let mut all_tools = vec![eth_tool];

        // Add contract-specific tools
        for contract in &config.contracts {
            let contract_tools = create_contract_tools(&contract);
            all_tools.extend(contract_tools);
        }

        // Add any custom tools provided by the caller
        if let Some(tools) = custom_tools {
            all_tools.extend(tools);
        }

        // Create the messages for the chat completion
        let mut messages = Vec::new();

        // Use existing messages from config if available
        if !config.messages.is_empty() {
            messages.extend(config.messages);
        } else {
            // If no messages in the config, add a default system message
            messages.push(Message {
                role: "system".into(),
                content: Some(
                    "You are an agent responsible for making and executing transactions.".into(),
                ),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            });
        }

        // Add the new user message
        messages.push(Message {
            role: "user".into(),
            content: Some(prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        // Call the LLM client with all tools
        let response = self.chat_completion(messages, Some(all_tools))?;

        // Check if we have tool calls
        if let Some(tool_calls) = response.tool_calls.clone() {
            if !tool_calls.is_empty() {
                // Use the tools module to process tool calls
                let tool_builder = crate::tools::ToolsBuilderImpl;

                // Call the tools builder to process tool calls
                // Here we need to adapt our types to the function signatures
                let result = tool_builder.process_tool_calls(
                    client::LlmClient::new(self.clone()),
                    Vec::new(), // Initial messages not used in our implementation
                    response.clone(),
                    tool_calls,
                    custom_handlers,
                );

                // Parse the tool result
                match result {
                    Ok(tool_result) => {
                        // Try to parse as a transaction
                        if let Ok(transaction) = serde_json::from_str::<
                            crate::bindings::exports::wavs::agent::types::Transaction,
                        >(&tool_result)
                        {
                            return Ok(LlmResponse::Transaction(transaction));
                        } else {
                            return Ok(LlmResponse::Text(tool_result));
                        }
                    }
                    Err(e) => {
                        return Ok(LlmResponse::Text(format!(
                            "Error processing tool calls: {}",
                            e
                        )));
                    }
                }
            }
        }

        // If we have content, return it as text response
        if let Some(content) = response.content {
            if !content.trim().is_empty() {
                return Ok(LlmResponse::Text(content));
            }
        }

        // No tool calls or content means no action needed
        Ok(LlmResponse::Text("".into()))
    }
}

// Clone implementation for LlmClientImpl
impl Clone for LlmClientImpl {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            config: self.config.clone(),
            api_url: self.api_url.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

// Helper function to create send ETH tool
fn create_send_eth_tool() -> Tool {
    Tool {
        tool_type: "function".into(),
        function: crate::bindings::exports::wavs::agent::types::Function {
            name: "send_eth".into(),
            description: Some("Send ETH to an address".into()),
            parameters: Some(
                r#"{
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Ethereum address to send ETH to"
                    },
                    "value": {
                        "type": "string",
                        "description": "Amount of ETH to send in wei"
                    }
                },
                "required": ["to", "value"]
            }"#
                .into(),
            ),
        },
    }
}

// Helper function to create tools for a contract
fn create_contract_tools(
    contract: &crate::bindings::exports::wavs::agent::types::Contract,
) -> Vec<Tool> {
    // Use the tools module to create tools from contract
    crate::tools::ToolsBuilderImpl.tools_from_contract(contract.clone())
}

// Add a conversion from serde_json::Error to AgentError
impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::Other(format!("JSON error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::exports::wavs::agent::client::GuestLlmClient;
    use crate::bindings::exports::wavs::agent::types::{LlmOptions, Message};

    #[test]
    fn test_client_init() {
        let client_impl = LlmClientImpl {
            model: "test-model".into(),
            config: LlmOptions {
                temperature: 0.7,
                top_p: 1.0,
                seed: 0,
                max_tokens: None,
                context_window: None,
            },
            api_url: "http://test".into(),
            api_key: None,
        };

        assert_eq!(client_impl.get_model(), "test-model");
        assert_eq!(client_impl.get_config().temperature, 0.7);
    }

    #[test]
    fn test_message_creation() {
        // Test that we can create a valid message
        let system_message = Message {
            role: "system".into(),
            content: Some("You are a helpful assistant".into()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        assert_eq!(system_message.role, "system");
        assert_eq!(system_message.content, Some("You are a helpful assistant".into()));
    }

    #[test]
    fn test_request_json_formation() {
        // Create a client
        let client_impl = LlmClientImpl {
            model: "gpt-4".into(),
            config: LlmOptions {
                temperature: 0.5,
                top_p: 0.9,
                seed: 42,
                max_tokens: Some(500),
                context_window: None,
            },
            api_url: "https://api.openai.com/v1/chat/completions".into(),
            api_key: Some("test-key".into()),
        };

        // Create messages (just for test completeness)
        let _messages = vec![
            Message {
                role: "system".into(),
                content: Some("You are a helpful assistant".into()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            Message {
                role: "user".into(),
                content: Some("Hello, how are you?".into()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        // This test just verifies request formation logic by checking client properties
        assert_eq!(client_impl.get_model(), "gpt-4");
        assert_eq!(client_impl.get_config().temperature, 0.5);

        // Verify tool creation
        let eth_tool = create_send_eth_tool();
        assert_eq!(eth_tool.tool_type, "function");
        assert_eq!(eth_tool.function.name, "send_eth");
    }
}
