# [WAVS](https://docs.wavs.xyz) Safe Template

**Template for getting started with developing WAVS applications and Gnosis Safe. NOT PRODUCTION READY.**

TODO:

- [ ] Refactor agent for better tool use and openai support
- [ ] No environment variables, use JSON
- [ ] Consolidate / refactor service types
- [ ] Better way to check result from Guard?

Later:

- [ ] Consider better guards like borg-core?

Contains WAVS enabled Safe Module and Guard contracts.

Reading and Resources:

- [Zodiac](https://www.zodiac.wiki/documentation): a bunch of useful extensions to the Safe. If you're looking for examples of extending Safe, Zodiac has a ton of them.
- [Safe Modules](https://docs.safe.global/advanced/smart-account-modules): documentation on Safe Modules, allowing easily extending functionality of a Safe.
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

## Create Project

```bash
# If you don't have foundry: `curl -L https://foundry.paradigm.xyz | bash && $HOME/.foundry/bin/foundryup`
forge init --template Lay3rLabs/wavs-foundry-template my-wavs --branch 0.3
```

> [!TIP]
> Run `make help` to see all available commands and environment variable overrides.

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

Now build the WASI rust components into the `compiled` output directory.

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

### Execute WASI component directly

Test run the component locally to validate the business logic works. An ID of 1 is Bitcoin. Nothing will be saved on-chain, just the output of the component is shown. This input is formatted using `cast format-bytes32-string` in the makefile command.

```bash
COIN_MARKET_CAP_ID=1 make wasi-exec
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

### Start Environment

Start an Ethereum node (anvil), the WAVS service, and deploy [eigenlayer](https://www.eigenlayer.xyz/) contracts to the local network.

```bash
cp .env.example .env

# Start the backend
#
# This must remain running in your terminal. Use another terminal to run other commands.
# You can stop the services with `ctrl+c`. Some MacOS terminals require pressing it twice.
make start-all
```

## WAVS Safe Module

A custom Safe module that integrates with WAVS.

### Deploy contracts

```bash
forge script script/WavsSafeModule.s.sol:Deploy --rpc-url http://localhost:8545 --broadcast

# Load the created addresses into the environment
export WAVS_SAFE_MODULE=$(cat .env | grep WAVS_SAFE_MODULE | tail -1 | cut -d '=' -f 2)
export WAVS_TRIGGER=$(cat .env | grep WAVS_TRIGGER | tail -1 | cut -d '=' -f 2)
# fish shell:
# set -gx WAVS_SAFE_MODULE (cat .env | grep WAVS_SAFE_MODULE | tail -1 | cut -d '=' -f 2)
# set -gx WAVS_TRIGGER (cat .env | grep WAVS_TRIGGER | tail -1 | cut -d '=' -f 2)
```

### Deploy service component

```bash
COMPONENT_FILENAME=dao_agent.wasm SERVICE_TRIGGER_ADDR=$WAVS_TRIGGER SERVICE_SUBMISSION_ADDR=$WAVS_SAFE_MODULE make deploy-service
```

### Trigger the AVS to execute a transaction

```bash
forge script script/WavsSafeModule.s.sol:AddTrigger --sig "run(string)" "We should donate 1 ETH to 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3." --rpc-url http://localhost:8545 --broadcast
```

### Check the balance

```bash
forge script script/WavsSafeModule.s.sol:ViewBalance --rpc-url http://localhost:8545
```

> Notice that the balance now contains the 1 ETH donation. If you don't see anything, watch the Anvil and WAVS logs during the trigger creation above to make sure the transaction is succeeding.

## WAVS Safe Guard

A custom Safe Guard that leverages WAVS to check whether transactions are authorized.

### Deploy contracts

```bash
forge script script/WavsSafeGuard.s.sol:Deploy --rpc-url http://localhost:8545 --broadcast

# Load the created addresses into the environment
export SAFE_ADDRESS=$(cat .env | grep SAFE_ADDRESS | tail -1 | cut -d '=' -f 2)
export GUARD_ADDRESS=$(cat .env | grep GUARD_ADDRESS | tail -1 | cut -d '=' -f 2)
# fish shell:
# set -gx SAFE_ADDRESS (cat .env | grep SAFE_ADDRESS | tail -1 | cut -d '=' -f 2)
# set -gx GUARD_ADDRESS (cat .env | grep GUARD_ADDRESS | tail -1 | cut -d '=' -f 2)
```

### Deploy service component

```bash
COMPONENT_FILENAME=safe_guard.wasm SERVICE_TRIGGER_ADDR=$SAFE_ADDRESS SERVICE_SUBMISSION_ADDR=$GUARD_ADDRESS TRIGGER_EVENT="ApproveHash(bytes32,address)" make deploy-service
```

### Trigger the validation process

```bash
forge script script/WavsSafeGuard.s.sol:ApproveSafeTransaction --rpc-url http://localhost:8545 --broadcast
```

### Execute the transaction

```bash
forge script script/WavsSafeGuard.s.sol:ExecuteSafeTransaction --rpc-url http://localhost:8545 --broadcast
```
