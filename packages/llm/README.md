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

TODO
