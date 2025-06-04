# [WAVS](https://docs.wavs.xyz) Safe Example

Contains WAVS-enabled Safe Module and Guard contracts, as well as a DEFINITELY NOT PRODUCTION-ready agent which controls the custom Safe Module.

The DAO Agent WAVS component leverages deterministic inferencing. See the [DETERMINISM.md](./DETERMINISM.md) file for more notes on making deterministic agents and the nuances involved.

Related Safe Resources:

- [Safe Modules](https://docs.safe.global/advanced/smart-account-modules): documentation on Safe Modules, allowing easy extension of Safe functionality.
- [Safe Guard](https://docs.safe.global/advanced/smart-account-guards): documentation on Safe Guards, allowing for checks on Safe transactions.

## System Requirements

<details>
<summary>Core (Docker, Compose, Make, JQ, Node v21+, Foundry)</summary>

## Ubuntu Base

- **Linux**: `sudo apt update && sudo apt install build-essential`

### Docker

If prompted, remove containerd with `sudo apt remove containerd.io`.

- **MacOS**: `brew install --cask docker`
- **Linux**: `sudo apt -y install docker.io`
- **Windows WSL**: [docker desktop wsl](https://docs.docker.com/desktop/wsl/#turn-on-docker-desktop-wsl-2) & `sudo chmod 666 /var/run/docker.sock`
- [Docker Documentation](https://docs.docker.com/get-started/get-docker/)

### Docker Compose

- **MacOS**: Already installed with Docker installer
  > `sudo apt remove docker-compose-plugin` may be required if you get a `dpkg` error
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

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
nvm install --lts
```

### Foundry

```bash docci-ignore
curl -L https://foundry.paradigm.xyz | bash && $HOME/.foundry/bin/foundryup
```

</details>

<details>

<summary>Rust v1.85+</summary>

### Rust Installation

```bash docci-ignore
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup toolchain install stable
rustup target add wasm32-wasip2
```

### Upgrade Rust

```bash docci-ignore
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

On Ubuntu LTS, if you later encounter errors like:

```bash
wkg: /lib/x86_64-linux-gnu/libm.so.6: version `GLIBC_2.38' not found (required by wkg)
wkg: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by wkg)
```

If GLIB is out of date. Consider updating your system using:

```bash
sudo do-release-upgrade
```

```bash docci-ignore
# Install required cargo components
# https://github.com/bytecodealliance/cargo-component#installation
cargo install cargo-binstall
cargo binstall cargo-component wasm-tools warg-cli wkg --locked --no-confirm --force

# Configure default registry
# Found at: $HOME/.config/wasm-pkg/config.toml
wkg config --default-registry wa.dev

# Allow publishing to a registry
warg key new
```

</details>

<details>
<summary>Install Ollama (optional)</summary>

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

## Build WASI components

Now build the WASI components into the `compiled` output directory.

> \[!WARNING]
> If you get: `error: no registry configured for namespace "wavs"`
>
> run, `wkg config --default-registry wa.dev`

> \[!WARNING]
> If you get: `failed to find the 'wasm32-wasip1' target and 'rustup' is not available`
>
> `brew uninstall rust` & install it from <https://rustup.rs>

```bash
# Remove `WASI_BUILD_DIR` to build all components.
make wasi-build
```

## WAVS

> \[!NOTE]
> If you are running on a Mac with an ARM chip, you will need to do the following:
>
> - Set up Rosetta: `softwareupdate --install-rosetta`
> - Enable Rosetta (Docker Desktop: Settings -> General -> enable "Use Rosetta for x86_64/amd64 emulation on Apple Silicon")
>
> Configure one of the following networking:
>
> - Docker Desktop: Settings -> Resources -> Network -> 'Enable Host Networking'
> - `brew install chipmk/tap/docker-mac-net-connect && sudo brew services start chipmk/tap/docker-mac-net-connect`

## Start Environment

Start an ethereum node (anvil), the WAVS service, and deploy [eigenlayer](https://www.eigenlayer.xyz/) contracts to the local network.

### Enable Telemetry (optional)

Set Log Level:

- Open the `.env` file.
- Set the `log_level` variable for wavs to debug to ensure detailed logs are captured.

> \[!NOTE]
> To see details on how to access both traces and metrics, please check out [Telemetry Documentation](telemetry/telemetry.md).

### Start the backend

```bash docci-background docci-delay-after=5
# This must remain running in your terminal. Use another terminal to run other commands.
# You can stop the services with `ctrl+c`. Some MacOS terminals require pressing it twice.
cp .env.example .env

# update the .env for either LOCAL or TESTNET

# Starts anvil + IPFS, WARG, Jaeger, and prometheus.
make start-all-local
```

## Create Deployer, upload Eigenlayer

These sections can be run on the **same** machine, or separate for testnet environments. Run the following steps on the deployer/aggregator machine.

```bash
# local: create deployer & auto fund. testnet: create & iterate check balance
bash ./script/create-deployer.sh

## Deploy Eigenlayer from Deployer
COMMAND=deploy make wavs-middleware
```

## Deploy Contracts and Service

**Key Concepts:**

- **Trigger Contract:** Any contract that emits events, then WAVS monitors. When a relevant event occurs, WAVS triggers the execution of your WebAssembly component.
- **Submission Contract:** This contract is used by the AVS service operator to submit the results generated by the WAVS component on-chain.

`WAVS_SERVICE_MANAGER_ADDRESS` is the address of the Eigenlayer service manager contract. It was deployed in the previous step. Then you deploy the trigger and submission contracts which depends on the service manager. The service manager will verify that a submission is valid (from an authorized operator) before saving it to the blockchain. The trigger contract is any arbitrary contract that emits some event that WAVS will watch for. Yes, this can be on another chain (e.g. an L2) and then the submission contract on the L1 _(Ethereum for now because that is where Eigenlayer is deployed)_.

### WAVS Safe Module + Agent Demo

A custom Safe module that integrates with WAVS.

```bash docci-delay-per-cmd=2
make deploy-safe-module-contracts
```

This will deploy both the WavsSafeModule, Trigger, and MockUSDC contracts, and write their addresses to a JSON file in the `.docker/module_deployments.json` path. The Trigger contract is meant to serve as an example; this agent could be triggered by other smart contract events.

Deploy the compiled component with the contract information from the previous steps.

```bash docci-delay-per-cmd=3
# ** Testnet Setup: https://wa.dev/account/credentials

export PKG_VERSION="0.4.0-rc.1"
COMPONENT_FILENAME=dao_agent.wasm PKG_NAME="dao-agent" source script/upload-to-wasi-registry.sh || true

# Testnet: set values (default: local if not set)
# export TRIGGER_CHAIN=holesky
# export SUBMIT_CHAIN=holesky

# Package not found with wa.dev? -- make sure it is public
export AGGREGATOR_URL=http://127.0.0.1:8001
REGISTRY=${REGISTRY} DEMO="module" bash ./script/build_service.sh
```

### WAVS Safe Guard Demo

A custom Safe Guard that leverages WAVS to check whether transactions are authorized.

```bash docci-delay-per-cmd=2
make deploy-safe-guard-contracts
```

This will deploy the Safe and Guard contracts, and write their addresses to a JSON file in the `.docker/guard_deployments.json` path.

> **IMPORTANT**: The guard needs to be explicitly enabled on the Safe to work. The Deploy script will attempt to enable it automatically, but this may not succeed if your Safe has a threshold > 1. The script will log information about what's happening.

Deploy the compiled component with the contract information from the previous steps.

```bash docci-delay-per-cmd=3
# ** Testnet Setup: https://wa.dev/account/credentials

export PKG_VERSION="0.4.0-rc.1"
COMPONENT_FILENAME=safe_guard.wasm PKG_NAME="safe-guard" source script/upload-to-wasi-registry.sh || true

# Testnet: set values (default: local if not set)
# export TRIGGER_CHAIN=holesky
# export SUBMIT_CHAIN=holesky

# Package not found with wa.dev? -- make sure it is public
export AGGREGATOR_URL=http://127.0.0.1:8001
REGISTRY=${REGISTRY} DEMO="guard" bash ./script/build_service.sh
```

## Upload to IPFS

```bash
# local
export DEPLOYER_PK=$(cat .nodes/deployer)
export WAVS_SERVICE_MANAGER_ADDRESS=$(jq -r .addresses.WavsServiceManager ./.nodes/avs_deploy.json)

# Upload service.json to IPFS
SERVICE_FILE=.docker/service.json source ./script/ipfs-upload.sh
```

## Start Aggregator

**TESTNET** You can move the aggregator it to its own machine for testnet deployments, it's easiest to run this on the deployer machine first. If moved, ensure you set the env variables correctly (copy pasted from the previous steps on the other machine).

```bash
bash ./script/create-aggregator.sh 1

IPFS_GATEWAY=${IPFS_GATEWAY} bash ./infra/aggregator-1/start.sh

wget -q --header="Content-Type: application/json" --post-data="{\"uri\": \"${IPFS_URI}\"}" ${AGGREGATOR_URL}/register-service -O -
```

## Start WAVS

**TESTNET** The WAVS service should be run in its own machine (creation, start, and opt-in). If moved, make sure you set the env variables correctly (copy pasted from the previous steps on the other machine).

```bash
bash ./script/create-operator.sh 1
```

Set the following environment variables in the `./infra/wavs-1/.env` file:

- `WAVS_ENV_OPENAI_API_KEY`: Your OpenAI API key for accessing LLM services
- `WAVS_ENV_OPENAI_API_URL`: The endpoint URL for OpenAI API calls (defaults to "https://api.openai.com/v1/chat/completions")
- `WAVS_ENV_IPFS_GATEWAY_URL`: IPFS gateway URL for loading configurations (defaults to "https://gateway.lighthouse.storage")

Example configuration in your `.env` file:

```
WAVS_ENV_OPENAI_API_KEY=sk-your-openai-key
WAVS_ENV_OPENAI_API_URL="https://api.openai.com/v1/chat/completions"
WAVS_ENV_IPFS_GATEWAY_URL="https://gateway.lighthouse.storage"
```

```bash
IPFS_GATEWAY=${IPFS_GATEWAY} bash ./infra/wavs-1/start.sh

# Deploy the service JSON to WAVS so it now watches and submits.
# 'opt in' for WAVS to watch (this is before we register to Eigenlayer)
WAVS_ENDPOINT=http://127.0.0.1:8000 SERVICE_URL=${IPFS_URI} IPFS_GATEWAY=${IPFS_GATEWAY} make deploy-service
```

## Register service specific operator

Making test mnemonic: `cast wallet new-mnemonic --json | jq -r .mnemonic`

Each service gets their own key path (hd_path). The first service starts at 1 and increments from there. Get the service ID

```bash
source ./script/avs-signing-key.sh

# TESTNET: set WAVS_SERVICE_MANAGER_ADDRESS
COMMAND="register ${OPERATOR_PRIVATE_KEY} ${AVS_SIGNING_ADDRESS} 0.001ether" make wavs-middleware

# Verify registration
COMMAND="list_operator" PAST_BLOCKS=500 make wavs-middleware
```

## Trigger the Service

### WAVS Safe Module

#### Trigger the AVS to execute a transaction

Test sending ETH:

```bash
forge script script/WavsSafeModule.s.sol:AddTrigger --sig "run(string)" "We should donate 1 ETH to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3." --rpc-url http://localhost:8545 --broadcast
```

Test sending an ERC20:

```bash
forge script script/WavsSafeModule.s.sol:AddTrigger --sig "run(string)" "We should donate 1 USDC to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3." --rpc-url http://localhost:8545 --broadcast
```

The script will automatically read the Trigger and MockUSDC contract addresses from the JSON file.

#### Check the balance

```bash
forge script script/WavsSafeModule.s.sol:ViewBalance --rpc-url http://localhost:8545
```

> Notice that the balance now contains both the 1 ETH and 1 USDC donations. If you don't see anything, watch the Anvil and WAVS logs during the trigger creation above to make sure the transaction is succeeding.

### WAVS Safe Guard

#### Try Executing a Transaction

Executing a Safe Transaction will fail as the AVS has not approved the transaction.

```bash
forge script script/WavsSafeGuard.s.sol:ExecuteSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

#### Trigger the validation process

Approve a Safe transaction for 0.1 ETH to `0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3`. The script will automatically read the Safe address from the JSON file.

```bash
forge script script/WavsSafeGuard.s.sol:ApproveSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

WAVS operators will pick up this event and run the `safe-gaurd` component, which currently always returns `true`. Note, this is meant as an example, you can extend it with custom application logic. After the AVS runs, the transaction will now be executable.

#### Execute the transaction

Now the Safe transaction is able to be executed.

```bash
forge script script/WavsSafeGuard.s.sol:ExecuteSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

If the Guard is properly enabled, this should only succeed if the transaction was previously approved by the WAVS service.

#### Testing the Guard Protection

To verify your guard is working properly:

1. Deploy a fresh Safe and Guard
2. Ensure the guard is properly enabled (verify with getGuard())
3. Skip the approval step
4. Try to execute a transaction directly
5. The transaction should fail with `AsyncValidationRequired` error

If the transaction succeeds without approval, your guard is not enabled properly.

Check balance, should be 0.1 ETH higher if transaction succeeded:

```bash
cast balance 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3 --rpc-url http://localhost:8545
```

# Claude Code

To spin up a sandboxed instance of [Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) in a Docker container that only has access to this project's files, run the following command:

```bash docci-ignore
npm run claude-code
# or with no restrictions (--dangerously-skip-permissions)
npm run claude-code:unrestricted
```
