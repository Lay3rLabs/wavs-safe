## WAVS Safe Module + Eigen Demo

This example deploys a WAVS Safe Module, that calls a function the strategy manger based on burn events from that same strategy manager.

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

# Deploy the service
COMPONENT_FILENAME=eigen_example.wasm SERVICE_TRIGGER_ADDR=$TRIGGER_ADDR SERVICE_SUBMISSION_ADDR=$MODULE_ADDR TRIGGER_EVENT="BurnableSharesIncreased(address,uint256)" make deploy-service
```

### Trigger the AVS to execute a transaction

Test Triggering:

```bash
# Load the Trigger address from the deployments file
TRIGGER_ADDR=$(jq -r '.triggerContract' .docker/module_deployments.json)

# Set strategy address and shares amount
export TRIGGER_ADDRESS=$TRIGGER_ADDR
export STRATEGY_ADDRESS=0x1234567890123456789012345678901234567890
export BURN_SHARES_AMOUNT=100

# Run the trigger burn shares script
forge script script/TriggerOperations.s.sol:TriggerBurnShares \
  --rpc-url http://localhost:8545 \
  --broadcast \
  -vvv
```

### Check the balance

```bash
# Query shares burned for this strategy
cast call $TRIGGER_ADDR "getSharesBurned(address)(uint256)" $STRATEGY_ADDR --rpc-url http://localhost:8545

# Query total shares burned
cast call $TRIGGER_ADDR "totalSharesBurned()(uint256)" --rpc-url http://localhost:8545
```
