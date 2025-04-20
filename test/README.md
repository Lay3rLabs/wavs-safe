# WAVS Safe Guard Tests

This directory contains the tests for the WavsSafeGuard contract, which is a Gnosis Safe guard that integrates with the WAVS (Web3 Autonomous Verification Service) for asynchronous transaction approval.

## Test Coverage

The tests validate that the WavsSafeGuard contract correctly enforces the asynchronous validation flow:

1. `testCheckTransactionWithoutValidation`: Verifies that transactions cannot be executed without prior async validation
2. `testCheckTransactionWithApproval`: Verifies that transactions can be executed after receiving async approval
3. `testCheckTransactionWithRejection`: Verifies that transactions are rejected if the async validation is denied
4. `testCheckTransactionWithExpiry`: Verifies that approved transactions cannot be executed after the validation timeout
5. `testTransactionStatusRetrieval`: Verifies the status retrieval functionality for transactions
6. `testCheckAfterExecution`: Verifies the post-execution checks
7. `testUnauthorizedCalls`: Verifies that only the Safe can call the guard functions
8. `testSupportsInterface`: Verifies ERC-165 interface detection

## Running the Tests

To run the tests, use the following command:

```bash
forge test
```

For more verbose output, use:

```bash
forge test -vv
```

## Test Design

The tests use mock contracts for the Safe and the WAVS Service Manager to simulate the behavior of these components. This approach allows isolated testing of the WavsSafeGuard contract without dependencies on external contracts.

### MockSafe

Simulates a Gnosis Safe with the necessary functions to test the WavsSafeGuard contract:

- Nonce management
- Transaction hash calculation

### MockServiceManager

Simulates the WAVS Service Manager with a simple validation function that just checks the presence of data and signature.

## Test Approach

The tests focus on direct calls to the WavsSafeGuard contract's functions to verify that:

1. Unauthorized calls are rejected
2. Transactions require async validation before execution
3. Validation has the correct expiration behavior
4. Status retrieval works correctly

This approach ensures that the guard's core functionality - preventing execution of transactions that haven't been approved through the async flow - works correctly.
