# WAVS LLM WASM Component

## Overview

The WAVS LLM WASM Component is a WebAssembly component that enables AI-assisted interactions with smart contracts and blockchain networks. It provides a modular architecture for integrating large language models into applications that need to interpret user requests, execute transactions, and interact with custom tools. As a WebAssembly component, it can be used across multiple programming languages including Rust, Go, and TypeScript.

## Features

- **LLM Integration**: Connects to language models from providers like OpenAI and Ollama
- **Smart Contract Interaction**: Automatically generates tools for smart contract functions
- **Transaction Construction**: Creates properly formatted transaction payloads
- **ABI Encoding**: Handles Ethereum ABI encoding for function calls
- **Extensible Tools System**: Supports custom tools for additional functionality
- **Configurable**: Flexible JSON configuration for customizing behavior

## Architecture

The WAVS LLM WASM Component consists of several key modules:

- **Client**: Handles communication with LLM providers, managing API requests and response processing
- **Config**: Manages component configuration, including contract definitions and LLM settings
- **Contracts**: Processes smart contract ABIs and handles transaction creation
- **Encoding**: Provides ABI encoding for Ethereum function calls
- **Tools**: Manages the tools system, including contract function calls and custom tools
- **Serialization**: Handles serialization/deserialization of WIT types

## Usage

### Setup

To use the WAVS LLM WASM Component in your project:

```rust
// Import the necessary types
use wavs_llm::bindings::exports::wavs::agent::{
    client::{GuestLlmClient, LlmClient},
    config::GuestConfigManager,
    tools::GuestToolsBuilder,
    types::{Config, LlmOptions, LlmResponse, Message, Tool},
};

// Create component instances
let config_manager = ConfigManagerImpl::new();
let llm_client = LlmClientImpl {
    model: "gpt-4".into(),
    config: LlmOptions {
        temperature: 0.7,
        top_p: 1.0,
        seed: 0,
        max_tokens: None,
        context_window: None,
    },
    api_url: String::new(),
    api_key: None,
};
let tools_builder = ToolsBuilderImpl;
```

### Processing User Prompts

```rust
// Load configuration
let config = match config_manager.load() {
    Ok(config) => config,
    Err(e) => {
        println!("Error loading config: {}", e);
        config_manager.default_config()
    }
};

// Initialize LLM client
let client = llm_client.new("gpt-4".to_string())?;

// Process user prompt
let prompt = "Transfer 0.1 ETH to 0x1234567890123456789012345678901234567890";
let response = client.process_prompt(
    prompt.to_string(),
    config,
    None, // Custom tools
    None, // Custom handlers
)?;

// Handle the response
match response {
    LlmResponse::Transaction(tx) => {
        println!("Transaction to execute:");
        println!("  To: {}", tx.to);
        println!("  Value: {}", tx.value);
        println!("  Data: {}", tx.data);
        // Execute the transaction...
    }
    LlmResponse::Text(text) => {
        println!("LLM response: {}", text);
    }
}
```

### Creating Custom Tools

```rust
// Create a custom tool
let custom_tool = tools_builder.custom_tool(
    "get_price".to_string(),
    "Get the current price of a token".to_string(),
    r#"{
        "type": "object",
        "properties": {
            "token": {
                "type": "string",
                "description": "The token symbol (e.g., ETH, BTC)"
            }
        },
        "required": ["token"]
    }"#.to_string()
);

// Implement a custom tool handler
struct PriceToolHandler;

impl CustomToolHandler {
    fn new() -> Self {
        Self {}
    }

    // Method to execute the tool call
    fn execute(&self, tool_call: ToolCall) -> Result<String, String> {
        // Parse arguments
        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse arguments: {}", e))?;

        // Extract token symbol
        let token = args["token"]
            .as_str()
            .ok_or("Missing token parameter")?;

        // In a real implementation, you would fetch the price from an API
        let price = match token.to_uppercase().as_str() {
            "ETH" => "1800.00",
            "BTC" => "60000.00",
            _ => "Unknown token",
        };

        Ok(format!("The current price of {} is ${}", token, price))
    }
}

// Create a custom handlers list
let custom_handlers = vec![PriceToolHandler::new()];
```

## Configuration

The component can be configured using a JSON configuration file:

```json
{
  "contracts": [
    {
      "name": "USDC",
      "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "abi": "[{\"constant\":false,\"inputs\":[{\"name\":\"_to\",\"type\":\"address\"},{\"name\":\"_value\",\"type\":\"uint256\"}],\"name\":\"transfer\",\"outputs\":[{\"name\":\"\",\"type\":\"bool\"}],\"type\":\"function\"}]",
      "description": "USDC is a stablecoin pegged to the US Dollar"
    }
  ],
  "llm_config": {
    "temperature": 0.0,
    "top_p": 1.0,
    "seed": 42,
    "max_tokens": 500,
    "context_window": 4096
  },
  "model": "gpt-4",
  "messages": [
    {
      "role": "system",
      "content": "You are an agent responsible for making and executing transactions."
    }
  ],
  "config": {
    "api_base_url": "https://api.openai.com/v1"
  }
}
```

## Environment Variables

The component uses the following environment variables:

- `config_uri`: URI to load the configuration from
- `WAVS_ENV_OPENAI_API_KEY`: API key for OpenAI models (only needed when using OpenAI models)
- `WAVS_ENV_OLLAMA_API_URL`: URL for Ollama API (default: http://localhost:11434)

## Example Prompts

```
Transfer 0.1 ETH to 0x1234...

Send 50 USDC to vitalik.eth

Call the vote function on the Governance contract with parameter true
```

## Using in a WebAssembly Host Environment

To use this component in a WebAssembly host environment, import the component using your language's WebAssembly Component Model tooling:

### Rust Example (using wasmtime)

```rust
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};

// Import the generated bindings for the component
use wavs_component::WavsLlmComponent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the engine
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    // Load the component
    let component = Component::from_file(&engine, "path/to/wavs_llm.wasm")?;

    // Create a store
    let mut store = Store::new(&engine, ());

    // Instantiate the component
    let (wavs, _) = WavsLlmComponent::instantiate(&mut store, &component)?;

    // Set up environment variables
    std::env::set_var("WAVS_ENV_OPENAI_API_KEY", "your-api-key");

    // Process a user request
    let prompt = "Transfer 0.1 ETH to 0x1234567890123456789012345678901234567890";
    let result = wavs.process_prompt(&mut store, prompt)?;

    println!("Result: {:?}", result);

    Ok(())
}
```

## Building Custom Applications

When building applications with this component, consider:

1. **Configuration Management**: Store your contract ABIs and configuration in a centralized location
2. **Error Handling**: Implement proper error handling for API failures and parsing errors
3. **Security**: Never expose API keys directly in your application
4. **Transaction Validation**: Always validate generated transactions before submitting them to a blockchain
5. **Custom Tools**: Implement domain-specific tools for your application's needs

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
