# [WAVS](https://docs.wavs.xyz) Safe Example

Contains WAVS-enabled Safe Module and Guard contracts, as well as a DEFINITELY NOT PRODUCTION-ready agent which controls the custom Safe Module.

The DAO Agent WAVS component leverages deterministic inferencing. See the [DETERMINISM.md](./DETERMINISM.md) file for more notes on making deterministic agents and the nuances involved.

Related Safe Resources:

- [Safe Modules](https://docs.safe.global/advanced/smart-account-modules): documentation on Safe Modules, allowing easy extension of Safe functionality.
- [Safe Guard](https://docs.safe.global/advanced/smart-account-guards): documentation on Safe Guards, allowing for checks on Safe transactions.

## System Requirements

<details>
<summary>Core (Docker, Compose, Make, JQ, Node v21+)</summary>

### Docker

- **MacOS**: `brew install --cask docker`
- **Linux**: `sudo apt -y install docker.io`
- **Windows WSL**: [docker desktop wsl](https://docs.docker.com/desktop/wsl/#turn-on-docker-desktop-wsl-2) & `sudo chmod 666 /var/run/docker.sock`
- [Docker Documentation](https://docs.docker.com/get-started/get-docker/)

### Docker Compose

- **MacOS**: Already installed with Docker installer
- **Linux + Windows WSL**: `sudo apt-get install docker-compose-v2`
- [Compose Documentation](https://docs.docker.com/compose/)

### Make

- **MacOS**: `brew install make`
- **Linux + Windows WSL**: `sudo apt -y install make`
- [Make Documentation](https://www.gnu.org/software/make/manual/make.html)

### JQ

- **MacOS**: `brew install jq`
- **Linux + Windows WSL**: `sudo apt -y install jq`
- [JQ Documentation](https://jqlang.org/download/)

### Node.js

- **Required Version**: v21+
- [Installation via NVM](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)
</details>

<details>

<summary>Rust v1.84+</summary>

### Rust Installation

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup toolchain install stable
rustup target add wasm32-wasip2
```

### Upgrade Rust

```bash
# Remove old targets if present
rustup target remove wasm32-wasi || true
rustup target remove wasm32-wasip1 || true

# Update and add required target
rustup update stable
rustup target add wasm32-wasip2
```

</details>

<details>
<summary>Cargo Components</summary>

### Install Cargo Components

```bash
# Install required cargo components
# https://github.com/bytecodealliance/cargo-component#installation
cargo install cargo-binstall
cargo binstall cargo-component warg-cli wkg --locked --no-confirm --force

# Configure default registry
wkg config --default-registry wa.dev
```

</details>

<details>
<summary>Install Ollama and Stable Diffusion</summary>
### Install Ollama (optional)

This example uses an LLM configured for determinism, run locally with Ollama. The model is llama3.2, but other open source models can be used if you change the model parameter in the config.

If you do not want to run a model locally, set `WAVS_ENV_OPENAI_API_KEY` with a valid OpenAI API key.

For more information about AVSs and deterministic AI, see our [blog post on the subject](https://www.layer.xyz/news-and-insights/deterministic-ai).

You can download Ollama here: https://ollama.com/

Get the llama 3.2 model.

```bash
ollama pull llama3.2
```

In a separate terminal run Ollama in the background with:

```bash
ollama serve
```

</details>

### Solidity

Install the required packages to build the Solidity contracts. This project supports both [submodules](./.gitmodules) and [npm packages](./package.json).

```bash
# Install packages (npm & submodules)
make setup

# Build the contracts
forge build

# Run the solidity tests
forge test
```

### Build WASI components

Now build the WASI Rust components into the `compiled` output directory.

> [!WARNING]
> If you get: `error: no registry configured for namespace "wavs"`
>
> run, `wkg config --default-registry wa.dev`

> [!WARNING]
> If you get: `failed to find the 'wasm32-wasip1' target and 'rustup' is not available`
>
> `brew uninstall rust` & install it from <https://rustup.rs>

```bash
make wasi-build # or `make build` to include solidity compilation.
```

## WAVS

> [!NOTE]
> If you are running on a Mac with an ARM chip, you will need to do the following:
>
> - Set up Rosetta: `softwareupdate --install-rosetta`
> - Enable Rosetta (Docker Desktop: Settings -> General -> enable "Use Rosetta for x86_64/amd64 emulation on Apple Silicon")
>
> Configure one of the following networking:
>
> - Docker Desktop: Settings -> Resources -> Network -> 'Enable Host Networking'
> - `brew install chipmk/tap/docker-mac-net-connect && sudo brew services start chipmk/tap/docker-mac-net-connect`

### Environment Variables and Key Values

WAVS components can access specific environment variables with the `WAVS_ENV_` prefix. These variables need to be:

1. Added to your local `.env` file
2. Listed in the `host_envs` array in the `SERVICE_CONFIG` when deploying the service
3. Used in your component code

For the DAO agent example, the following environment variables are used:

- `WAVS_ENV_OPENAI_API_KEY`: Your OpenAI API key for accessing LLM services
- `WAVS_ENV_OPENAI_API_URL`: The endpoint URL for OpenAI API calls (defaults to "https://api.openai.com/v1/chat/completions")
- `WAVS_ENV_IPFS_GATEWAY_URL`: IPFS gateway URL for loading configurations (defaults to "https://gateway.lighthouse.storage")

Example configuration in your `.env` file:

```
WAVS_ENV_OPENAI_API_KEY=sk-your-openai-key
WAVS_ENV_OPENAI_API_URL="https://api.openai.com/v1/chat/completions"
WAVS_ENV_IPFS_GATEWAY_URL="https://gateway.lighthouse.storage"
```

The Key Value pairs must be listed in the `kv` array in the `SERVICE_CONFIG` when you deploy a component. The `dao-agent` component supports a `config_uri` kv pair where it fetches the agent configuration (otherwise it uses the default config, which matches [agent-config.example.json](./agent-config.example.json)):

Example `host_envs` and `kv` in SERVICE_CONFIG:

```
SERVICE_CONFIG='{"fuel_limit":100000000,"max_gas":5000000,"host_envs":["WAVS_ENV_OPENAI_API_KEY", "WAVS_ENV_OPENAI_API_URL", "WAVS_ENV_IPFS_GATEWAY_URL"],"kv":[["config_uri", "ipfs://bafkreiaqticxepygpav5h52kcqtid3ls2mm55i2so7edxmrdbn3z3rnyny"]],"workflow_id":"default","component_id":"default"}'
```

### Start Environment

Start an Ethereum node (anvil), the WAVS service, and deploy [EigenLayer](https://www.eigenlayer.xyz/) contracts to the local network.

```bash
cp .env.example .env

# Start the backend
#
# This must remain running in your terminal. Use another terminal to run other commands.
# You can stop the services with `ctrl+c`. Some MacOS terminals require pressing it twice.
make start-all
```

### Execute WASI component directly

Test run the component locally to validate the business logic works (no on-chain interactions). Be sure to run `make wasi-build` if you make changes.

Note: the `SERVICE_CONFIG` json is used for setting environment variables and the kv store. The `kv` array is a list of key value pairs, "config_uri" is a URI that contains the agent config (the IPFS URI included below corresponds to `agent-config.example.json`). To use a different model, or change the agent configuration, you'll need to upload a new JSON somewhere (or modify the default context in `dao-agent/src/context.rs`).

```bash
COMPONENT_FILENAME="dao_agent.wasm" PROMPT='We should donate 1 ETH to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3.' SERVICE_CONFIG='{"fuel_limit":100000000,"max_gas":5000000,"host_envs":["WAVS_ENV_OPENAI_API_KEY", "WAVS_ENV_OPENAI_API_URL", "WAVS_ENV_IPFS_GATEWAY_URL"],"kv":[["config_uri", "ipfs://bafkreiaqticxepygpav5h52kcqtid3ls2mm55i2so7edxmrdbn3z3rnyny"]],"workflow_id":"default","component_id":"default"}' make wasi-exec
```

## WAVS Safe Module + Agent Demo

A custom Safe module that integrates with WAVS.

### Deploy contracts

```bash
forge script script/WavsSafeModule.s.sol:Deploy --rpc-url http://localhost:8545 --broadcast
```

This will deploy both the WavsSafeModule, Trigger, and MockUSDC contracts, and write their addresses to a JSON file in the `.docker/module_deployments.json` path. The Trigger contract is meant to serve as an example; this agent could be triggered by other smart contract events.

### Deploy service component

```bash
# Load the addresses from the JSON file
TRIGGER_ADDR=$(jq -r '.triggerContract' .docker/module_deployments.json)
MODULE_ADDR=$(jq -r '.wavsSafeModule' .docker/module_deployments.json)

# Set service config
SERVICE_CONFIG='{"fuel_limit":100000000,"max_gas":5000000,"host_envs":["WAVS_ENV_OPENAI_API_KEY", "WAVS_ENV_OPENAI_API_URL", "WAVS_ENV_IPFS_GATEWAY_URL"],"kv":[],"workflow_id":"default","component_id":"default"}'

# Deploy the service
COMPONENT_FILENAME=dao_agent.wasm SERVICE_TRIGGER_ADDR=$TRIGGER_ADDR SERVICE_SUBMISSION_ADDR=$MODULE_ADDR SERVICE_CONFIG=$SERVICE_CONFIG make deploy-service
```

### Trigger the AVS to execute a transaction

Test sending ETH:

```bash
forge script script/WavsSafeModule.s.sol:AddTrigger --sig "run(string)" "We should donate 1 ETH to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3." --rpc-url http://localhost:8545 --broadcast
```

Test sending an ERC20:

```bash
forge script script/WavsSafeModule.s.sol:AddTrigger --sig "run(string)" "We should donate 1 USDC to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3." --rpc-url http://localhost:8545 --broadcast
```

The script will automatically read the Trigger and MockUSDC contract addresses from the JSON file.

### Check the balance

```bash
forge script script/WavsSafeModule.s.sol:ViewBalance --rpc-url http://localhost:8545
```

> Notice that the balance now contains both the 1 ETH and 1 USDC donations. If you don't see anything, watch the Anvil and WAVS logs during the trigger creation above to make sure the transaction is succeeding.

## WAVS Safe Guard Demo

A custom Safe Guard that leverages WAVS to check whether transactions are authorized.

### Deploy contracts

```bash
forge script script/WavsSafeGuard.s.sol:Deploy --rpc-url http://localhost:8545 --broadcast
```

This will deploy the Safe and Guard contracts, and write their addresses to a JSON file in the `.docker/guard_deployments.json` path.

### Deploy service component

```bash
# Load the addresses from the JSON file
SAFE_ADDR=$(jq -r '.safeAddress' .docker/guard_deployments.json)
GUARD_ADDR=$(jq -r '.guardAddress' .docker/guard_deployments.json)

# Deploy the service
COMPONENT_FILENAME=safe_guard.wasm SERVICE_TRIGGER_ADDR=$SAFE_ADDR SERVICE_SUBMISSION_ADDR=$GUARD_ADDR TRIGGER_EVENT="ApproveHash(bytes32,address)" make deploy-service
```

### Trigger the validation process

```bash
forge script script/WavsSafeGuard.s.sol:ApproveSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

The script will automatically read the Safe address from the JSON file.

### Execute the transaction

```bash
forge script script/WavsSafeGuard.s.sol:ExecuteSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

The script will automatically read the Safe address from the JSON file.
