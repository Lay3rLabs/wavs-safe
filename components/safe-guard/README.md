# Safe Guard

## Overview

The Safe Guard component acts as a transaction validator for Gnosis Safe within the WAVS ecosystem. It provides critical security checks for transactions initiated by the DAO agent, ensuring that only valid and approved transactions are executed on-chain.

The Safe Guard is a crucial security layer that:
- Validates transaction hashes against approved operations
- Ensures transaction integrity before execution
- Applies configurable rule-based validation logic
- Prevents execution of unauthorized or malicious transactions
- Works alongside WavsSafeGuard.sol contract for on-chain execution control

This component is designed to run deterministically across all WAVS operators, ensuring consensus on transaction validity before execution occurs through the Gnosis Safe multi-signature wallet.

Related Safe Resources:

- [Safe Modules](https://docs.safe.global/advanced/smart-account-modules): documentation on Safe Modules, allowing easy extension of Safe functionality.
- [Safe Guard](https://docs.safe.global/advanced/smart-account-guards): documentation on Safe Guards, allowing for checks on Safe transactions.

## Implementation

The component listens for `ApproveHash` events from the Safe contract and makes decisions about whether to approve transactions:

```rust
// Event structure for transaction approval
event ApproveHash(bytes32 indexed approvedHash, address indexed owner);

// Response structure for validation
struct ValidationPayload {
    bytes32 approvedHash;
    bool approved;
}
```

It then returns a validation response that determines whether the transaction will be permitted to execute through the Gnosis Safe. This validation is critical for maintaining the security of DAO treasury operations that are initiated by AI-powered decision making.

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
