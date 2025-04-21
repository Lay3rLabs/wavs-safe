# DAO Agent

## Overview

The DAO Agent is an AI-powered autonomous agent for Gnosis Safe that can make financial decisions and execute transactions on behalf of a DAO. Built using WAVS (WebAssembly Autonomous Verification System), it combines the security of multi-signature wallets with the flexibility of AI-assisted decision making.

## Features

- **AI-Powered Decision Making**: Uses LLMs to interpret requests and decide on appropriate financial actions
- **ETH Transfers**: Securely sends ETH to allowed addresses
- **Smart Contract Interaction**: Calls functions on verified contracts like ERC20 tokens
- **Dynamic Balance Checking**: Queries on-chain balances for ETH and ERC20 tokens in real-time
- **Configurable Behavior**: Customizable through JSON configuration that can be loaded from HTTP or IPFS

## Configuration

The agent loads its configuration from a JSON file, which can be specified through the `config_uri` environment variable. Configuration can be hosted on HTTP/HTTPS or IPFS.

### Configuration Options

```json
{
  "account_address": "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
  "allowlisted_addresses": ["0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3"],
  "supported_tokens": [
    {
      "address": "0x0000000000000000000000000000000000000000",
      "symbol": "ETH",
      "decimals": 18,
      "description": "Native Ethereum token"
    },
    {
      "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "symbol": "USDC",
      "decimals": 6,
      "description": "USD Coin stablecoin"
    }
  ],
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
  "system_prompt_template": "..."
}
```

## Environment Variables

The agent requires the following environment variables:

- `ETH_RPC_URL`: The Ethereum RPC URL for querying on-chain data
- `ETH_CHAIN` (optional): The Ethereum chain to use (defaults to "sepolia")

## Dynamic Balance Checking

The agent now dynamically queries on-chain balances in real-time using the Ethereum JSON-RPC API. This ensures decisions are made based on the most up-to-date financial information. The agent:

1. Queries ETH balance directly from the blockchain
2. Retrieves ERC20 token balances by calling the `balanceOf` function on each token contract
3. Formats these balances for display in the system prompt

## Usage

The DAO Agent is designed to be triggered by events, typically from the Gnosis Safe Module. It can also be triggered directly for testing purposes.

### Example Prompts

```
Send 0.1 ETH to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3 for development expenses
```

```
Transfer 500 USDC to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3 for community grants
```

## License

This project is licensed under MIT License - see the LICENSE file for details.
