# WAVS Agent

## Overview

The WAVS Agent is a flexible AI-powered autonomous agent built using WebAssembly Autonomous Verification System (WAVS). It enables the creation of AI-assisted components that can interact with smart contracts, execute transactions, and integrate with custom tools through a modular architecture.

## Features

- **LLM-Powered Decision Making**: Uses large language models to interpret requests and decide on appropriate actions
- **Smart Contract Interaction**: Seamlessly interacts with any smart contract through ABI interfaces
- **Transaction Handling**: Generates properly formatted transaction payloads
- **Extensible Tooling**: Easily add custom tools beyond smart contract calls
- **Configurable Behavior**: Customizable through JSON configuration that can be loaded from HTTP or IPFS

## Architecture

The WAVS Agent is built around a few core components:

- **Context**: Stores configuration, contracts, messages, and system prompts. This simplified structure makes it easier to customize and extend the agent's capabilities without being tied to specific use cases like DAOs.

- **LLM Client**: Handles interactions with language models (supports OpenAI and Ollama). The client abstracts away the differences between various LLM providers, offering a consistent interface.

- **Tools System**: Provides a framework for creating and handling tools. The agent automatically generates tools for each smart contract function defined in the configuration, allowing for seamless blockchain interaction.

- **Contract Handling**: Automatically encodes function calls to smart contracts using their ABIs, handling the complexity of blockchain interactions for you.

- **Custom Tool Extension**: Allows adding arbitrary functionality via custom tools that can perform any operation, from API calls to complex calculations.

### Processing Flow

1. User sends a text prompt to the agent
2. Agent loads the context and initializes LLM client
3. Agent generates available tools from smart contracts and custom tools
4. Agent sends the prompt, context, and tools to the LLM
5. LLM decides on an action and may call one or more tools
6. Tool calls are processed by appropriate handlers
7. Result is returned as either a transaction or text response

## Configuration

The agent loads its configuration from a JSON file, which can be specified through the `config_uri` environment variable. Configuration can be hosted on HTTP/HTTPS or IPFS.

### Configuration Options

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
    "temperature": 0,
    "top_p": 0.1,
    "seed": 42,
    "max_tokens": 500,
    "context_window": 4096
  },
  "model": "llama3.2",
  "system_prompt": "You are an agent responsible for making and executing transactions. Use the available tools to interact with smart contracts.",
  "messages": []
}
```

## Usage as a Library

The WAVS Agent is designed to be used as a library in your WASI components. Here's a basic example of how to use it:

```rust
use components::wavs_agent::{
    context::Context,
    llm::{LLMClient, LlmResponse},
};
use wstd::runtime::block_on;

fn process_user_request(prompt: &str) -> Result<(), String> {
    block_on(async {
        // Load context
        let context = Context::load().await?;

        // Create LLM client
        let client = LLMClient::with_config(&context.model, context.llm_config.clone())?;

        // Process prompt
        let llm_response = client.process_prompt(prompt, &context, None, None).await?;

        // Handle the response
        match llm_response {
            LlmResponse::Transaction(tx) => {
                println!("Transaction to execute: {:?}", tx);
                // Your logic to handle the transaction
            },
            LlmResponse::Text(text) => {
                println!("LLM response: {}", text);
                // Your logic to handle the text response
            }
        }

        Ok(())
    })
}
```

## Using Custom Tools

One of the most powerful features of the WAVS Agent is the ability to add custom tools beyond smart contract interactions. Here's an example:

```rust
use components::wavs_agent::{
    context::Context,
    llm::{LLMClient, LlmResponse},
    tools::{builders, CustomToolHandler, Tool, ToolCall},
};
use serde_json::json;
use std::boxed::Box;

// 1. Create a custom tool definition
let weather_tool = builders::custom_tool(
    "get_weather",
    "Get the current weather for a location",
    json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "The city name or zip code"
            }
        },
        "required": ["location"]
    })
);

// 2. Create a custom tool handler
struct WeatherToolHandler;

impl CustomToolHandler for WeatherToolHandler {
    fn can_handle(&self, tool_name: &str) -> bool {
        tool_name == "get_weather"
    }

    fn execute(&self, tool_call: &ToolCall) -> Result<String, String> {
        // Parse the arguments
        let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| format!("Failed to parse arguments: {}", e))?;

        // Extract the location
        let location = args["location"].as_str().ok_or("Missing location")?;

        // In a real implementation, you would call a weather API here
        // For this example, we'll just return a static response
        let weather_info = format!("{{\"temperature\": 72, \"conditions\": \"sunny\", \"location\": \"{}\"}}", location);

        Ok(weather_info)
    }
}

// 3. Create a vector of custom tools and handlers
let custom_tools = vec![weather_tool];
let weather_handler = Box::new(WeatherToolHandler);
let custom_handlers: Vec<Box<dyn CustomToolHandler>> = vec![weather_handler];

// 4. Use the tools with the LLM client
async fn example() -> Result<(), String> {
    let context = Context::load().await?;
    let client = LLMClient::with_config("llama3.2", context.llm_config.clone())?;

    let prompt = "What's the weather like in San Francisco?";
    let result = client.process_prompt(
        prompt,
        &context,
        custom_tools,
        custom_handlers.as_ref().map(|h| h.as_slice())
    ).await?;

    match result {
        LlmResponse::Text(text) => println!("Response: {}", text),
        LlmResponse::Transaction(tx) => println!("Transaction: {:?}", tx),
    }

    Ok(())
}
```

### How Custom Tools Work

1. **Custom Tool Definition**: Use `builders::custom_tool()` to create a new tool with a name, description, and JSON Schema for parameters.

2. **Custom Tool Handler**: Implement the `CustomToolHandler` trait to handle calls to your custom tool:

   - `can_handle()`: Returns true if this handler can process the given tool name
   - `execute()`: Processes the tool call and returns a result string

3. **Integration**: Pass your custom tools and handlers to the `process_prompt` method:
   - `custom_tools`: A vector of Tool objects defining what tools are available
   - `custom_handlers`: A slice of boxed CustomToolHandler trait objects that implement the tool functionality

This approach allows you to extend the agent with any custom functionality you need, such as:

- API calls to external services
- Database queries
- File system operations
- Complex calculations or data processing
- Integration with other components or systems

## Environment Variables

- `config_uri`: URI to load the configuration from (supports HTTP/HTTPS or IPFS)
- `WAVS_ENV_IPFS_GATEWAY_URL`: IPFS gateway URL to use when loading from IPFS (default: https://gateway.lighthouse.storage/ipfs)
- `WAVS_ENV_OLLAMA_API_URL`: URL for Ollama API (default: http://localhost:11434)
- `WAVS_ENV_OPENAI_API_KEY`: API key for OpenAI models (required only when using OpenAI models)

## Example Prompts

```
Transfer 0.1 ETH to 0x1234...

Call the vote function on the Governance contract with parameter true
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Building Custom WASI Components

The WAVS Agent is designed to be a building block for creating custom WASI components. Here's how to create your own WASI component using the WAVS Agent:

### 1. Add Dependencies to Cargo.toml

```toml
[dependencies]
wavs-agent = { git = "https://github.com/wavslabs/wavs-agent.git" }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wstd = "0.1.0"
```

### 2. Implement the WASI Component

```rust
mod bindings;
use bindings::{export, Guest, TriggerAction};
use wavs_agent::{
    context::Context,
    llm::{LLMClient, LlmResponse},
    tools::{builders, CustomToolHandler, Tool, ToolCall},
};
use wstd::runtime::block_on;

struct Component;

impl Guest for Component {
    fn run(trigger_action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Extract the prompt from the trigger data
        let prompt = extract_prompt_from_trigger(&trigger_action)?;

        return block_on(async move {
            // Load context
            let context = Context::load().await?;

            // Create LLM client
            let client = LLMClient::with_config(&context.model, context.llm_config.clone())?;

            // Add any custom tools
            let custom_tools = create_custom_tools();
            let custom_handlers = create_custom_handlers();

            // Process prompt using LLM with tools
            let llm_response = client.process_prompt(
                &prompt,
                &context,
                custom_tools,
                custom_handlers.as_ref().map(|h| h.as_slice())
            ).await?;

            // Handle the response appropriately for your component
            match llm_response {
                LlmResponse::Transaction(tx) => {
                    // Process transaction based on your component's needs
                    // ...

                    // Return serialized data
                    Ok(Some(serde_json::to_vec(&tx).unwrap()))
                },
                LlmResponse::Text(text) => {
                    // Process text response
                    // ...

                    // Return text data
                    Ok(Some(text.as_bytes().to_vec()))
                }
            }
        });
    }
}

// Helper function to extract the prompt from trigger action
fn extract_prompt_from_trigger(trigger_action: &TriggerAction) -> Result<String, String> {
    use bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
    use alloy_sol_types::sol_data::String as SolString;
    use alloy_sol_types::SolType;

    match &trigger_action.data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            // Decode the ABI-encoded string from the event log
            let decoded = SolString::abi_decode(&log.data, false)
                .map_err(|e| format!("Failed to decode ABI string: {}", e))?;

            Ok(decoded.to_string())
        }
        TriggerData::Raw(data) => {
            // For raw data, just convert from UTF-8 bytes
            let prompt = std::str::from_utf8(data)
                .map_err(|e| format!("Failed to decode prompt from bytes: {}", e))?;

            Ok(prompt.to_string())
        }
        _ => Err("Unsupported trigger data".to_string()),
    }
}

export!(Component with_types_in bindings);
```

### 3. Create Custom Tools (Optional)

```rust
fn create_custom_tools() -> Option<Vec<Tool>> {
    // Define custom tools for your component
    let tools = vec![
        builders::custom_tool(
            "custom_action",
            "Perform a custom action",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "parameter": {
                        "type": "string",
                        "description": "Input parameter"
                    }
                },
                "required": ["parameter"]
            })
        )
    ];

    Some(tools)
}

fn create_custom_handlers() -> Option<Vec<Box<dyn CustomToolHandler>>> {
    // Create handlers for your custom tools
    let handlers: Vec<Box<dyn CustomToolHandler>> = vec![
        Box::new(CustomActionHandler)
    ];

    Some(handlers)
}

struct CustomActionHandler;

impl CustomToolHandler for CustomActionHandler {
    fn can_handle(&self, tool_name: &str) -> bool {
        tool_name == "custom_action"
    }

    fn execute(&self, tool_call: &ToolCall) -> Result<String, String> {
        // Implement your custom action here
        // ...

        Ok("Success!".to_string())
    }
}
```

This approach allows you to build specialized WASI components that leverage the WAVS Agent's capabilities while adding your own custom functionality.
