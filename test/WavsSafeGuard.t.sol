// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "@gnosis.pm/safe-contracts/contracts/common/Enum.sol";
import "../src/contracts/WavsSafeGuard.sol";
import {IWavsServiceManager} from "@wavs/interfaces/IWavsServiceManager.sol";

contract MockServiceManager is IWavsServiceManager {
    function validate(
        bytes calldata data,
        bytes calldata signature
    ) external pure override {
        // Simple mock implementation that just validates the function can be called
        // In a real scenario, this would validate signatures
        require(data.length > 0, "Empty data");
        require(signature.length > 0, "Empty signature");
    }
}

// Mock Safe contract for testing
contract MockSafe {
    uint256 private _nonce;

    constructor() {
        _nonce = 1; // Start with nonce 1 to match WavsSafeGuard behavior
    }

    function nonce() external view returns (uint256) {
        return _nonce;
    }

    function incrementNonce() external {
        _nonce++;
    }

    function getTransactionHash(
        address to,
        uint256 value,
        bytes memory data,
        Enum.Operation operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address refundReceiver,
        uint256 txNonce
    ) external pure returns (bytes32) {
        return
            keccak256(
                abi.encode(
                    to,
                    value,
                    keccak256(data),
                    operation,
                    safeTxGas,
                    baseGas,
                    gasPrice,
                    gasToken,
                    refundReceiver,
                    txNonce
                )
            );
    }
}

contract WavsSafeGuardTest is Test {
    // Contracts
    WavsSafeGuard guard;
    MockServiceManager serviceManager;
    MockSafe mockSafe;

    // Test accounts
    address owner;
    uint256 ownerKey;

    // Create validation payload structure
    // Since ValidationPayload is internal to WavsSafeGuard, we recreate its structure
    struct ValidationPayload {
        bytes32 approvedHash;
        bool approved;
    }

    function setUp() public {
        // Create test account
        (owner, ownerKey) = makeAddrAndKey("owner");

        // Deploy mock Safe
        mockSafe = new MockSafe();

        // Deploy mock service manager
        serviceManager = new MockServiceManager();

        // Deploy guard
        guard = new WavsSafeGuard(
            payable(address(mockSafe)),
            address(serviceManager)
        );
    }

    function testCheckTransactionWithoutValidation() public {
        // Transaction details
        address to = address(0x123);
        uint256 value = 0.1 ether;
        bytes memory data = "";
        Enum.Operation operation = Enum.Operation.Call;

        // Call the checkTransaction function directly (should revert with AsyncValidationRequired)
        vm.prank(address(mockSafe)); // Mock that the call comes from Safe
        vm.expectRevert(WavsSafeGuard.AsyncValidationRequired.selector);
        guard.checkTransaction(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            bytes(""), // signatures
            address(0) // msgSender
        );
    }

    function testCheckTransactionWithApproval() public {
        // Transaction details
        address to = address(0x123);
        uint256 value = 0.1 ether;
        bytes memory data = "";
        Enum.Operation operation = Enum.Operation.Call;

        // Calculate the transaction hash - use current nonce
        bytes32 txHash = mockSafe.getTransactionHash(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            address(0), // refundReceiver
            mockSafe.nonce() - 1 // Use nonce - 1 as WavsSafeGuard expects
        );

        // Simulate async validation approval
        simulateAsyncApproval(txHash, true);

        // Call the checkTransaction function directly (should not revert)
        vm.prank(address(mockSafe)); // Mock that the call comes from Safe
        guard.checkTransaction(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            bytes(""), // signatures
            address(0) // msgSender
        );

        // Verify transaction status
        (WavsSafeGuard.ValidationStatus status, uint256 remainingTime) = guard
            .getTransactionStatus(txHash);
        assertEq(uint(status), uint(WavsSafeGuard.ValidationStatus.Approved));
        assertGt(remainingTime, 0);
    }

    function testCheckTransactionWithRejection() public {
        // Transaction details
        address to = address(0x123);
        uint256 value = 0.1 ether;
        bytes memory data = "";
        Enum.Operation operation = Enum.Operation.Call;

        // Calculate the transaction hash - use current nonce
        bytes32 txHash = mockSafe.getTransactionHash(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            address(0), // refundReceiver
            mockSafe.nonce() - 1 // Use nonce - 1 as WavsSafeGuard expects
        );

        // Simulate async validation rejection
        simulateAsyncApproval(txHash, false);

        // Call the checkTransaction function directly (should revert with rejected message)
        vm.prank(address(mockSafe)); // Mock that the call comes from Safe
        vm.expectRevert(WavsSafeGuard.TransactionRejected.selector);
        guard.checkTransaction(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            bytes(""), // signatures
            address(0) // msgSender
        );
    }

    function testCheckTransactionWithExpiry() public {
        // Transaction details
        address to = address(0x123);
        uint256 value = 0.1 ether;
        bytes memory data = "";
        Enum.Operation operation = Enum.Operation.Call;

        // Calculate the transaction hash - use current nonce
        bytes32 txHash = mockSafe.getTransactionHash(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            address(0), // refundReceiver
            mockSafe.nonce() - 1 // Use nonce - 1 as WavsSafeGuard expects
        );

        // Simulate async validation approval
        simulateAsyncApproval(txHash, true);

        // Fast-forward time past the validation timeout (1 hour + 1 second)
        vm.warp(block.timestamp + 1 hours + 1 seconds);

        // Call the checkTransaction function directly (should revert with expired)
        vm.prank(address(mockSafe)); // Mock that the call comes from Safe
        vm.expectRevert(WavsSafeGuard.TransactionExpired.selector);
        guard.checkTransaction(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            bytes(""), // signatures
            address(0) // msgSender
        );
    }

    function testTransactionStatusRetrieval() public {
        // Transaction details
        address to = address(0x123);
        uint256 value = 0.1 ether;
        bytes memory data = "";
        Enum.Operation operation = Enum.Operation.Call;

        // Calculate the transaction hash - use current nonce
        bytes32 txHash = mockSafe.getTransactionHash(
            to,
            value,
            data,
            operation,
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            address(0), // refundReceiver
            mockSafe.nonce() - 1 // Use nonce - 1 as WavsSafeGuard expects
        );

        // Initially status should be NotExists
        (WavsSafeGuard.ValidationStatus status, uint256 remainingTime) = guard
            .getTransactionStatus(txHash);
        assertEq(uint(status), uint(WavsSafeGuard.ValidationStatus.NotExists));
        assertEq(remainingTime, 0);

        // Simulate async validation approval
        simulateAsyncApproval(txHash, true);

        // After approval, status should be Approved with remaining time > 0
        (status, remainingTime) = guard.getTransactionStatus(txHash);
        assertEq(uint(status), uint(WavsSafeGuard.ValidationStatus.Approved));
        assertGt(remainingTime, 0);

        // Fast-forward time to half of the validation timeout
        vm.warp(block.timestamp + 30 minutes);

        // Check remaining time has decreased
        (status, remainingTime) = guard.getTransactionStatus(txHash);
        assertEq(uint(status), uint(WavsSafeGuard.ValidationStatus.Approved));
        assertLt(remainingTime, 30 minutes + 5); // Allow for slight timestamp variations

        // Fast-forward time past the validation timeout
        vm.warp(block.timestamp + 31 minutes);

        // After expiry, status should still be Approved but remaining time should be 0
        (status, remainingTime) = guard.getTransactionStatus(txHash);
        assertEq(uint(status), uint(WavsSafeGuard.ValidationStatus.Approved));
        assertEq(remainingTime, 0);
    }

    function testCheckAfterExecution() public {
        // Call the checkAfterExecution function directly
        vm.prank(address(mockSafe)); // Mock that the call comes from Safe
        guard.checkAfterExecution(bytes32(0), true);

        // Call with failure should revert
        vm.prank(address(mockSafe));
        vm.expectRevert(WavsSafeGuard.TransactionFailed.selector);
        guard.checkAfterExecution(bytes32(0), false);
    }

    function testUnauthorizedCalls() public {
        // Try to call checkTransaction from unauthorized address
        vm.prank(address(0x456));
        vm.expectRevert(WavsSafeGuard.Unauthorized.selector);
        guard.checkTransaction(
            address(0x123),
            0.1 ether,
            "",
            Enum.Operation.Call,
            0,
            0,
            0,
            address(0),
            payable(address(0)),
            bytes(""),
            address(0)
        );

        // Try to call checkAfterExecution from unauthorized address
        vm.prank(address(0x456));
        vm.expectRevert(WavsSafeGuard.Unauthorized.selector);
        guard.checkAfterExecution(bytes32(0), true);
    }

    function testSupportsInterface() external view {
        // Test the supportsInterface function
        bytes4 guardInterfaceId = type(Guard).interfaceId;
        bool isSupported = guard.supportsInterface(guardInterfaceId);
        assertTrue(isSupported);

        // Test with a different interface ID
        bytes4 randomInterfaceId = bytes4(keccak256("random"));
        isSupported = guard.supportsInterface(randomInterfaceId);
        assertFalse(isSupported);
    }

    // Helper to simulate the async validation process
    function simulateAsyncApproval(bytes32 txHash, bool approved) internal {
        // Create and encode the validation payload
        ValidationPayload memory payload = ValidationPayload({
            approvedHash: txHash,
            approved: approved
        });

        bytes memory data = abi.encode(payload);

        // Create a mock signature
        bytes memory signature = abi.encode("mock_signature");

        // Call handleSignedData to update the validation status
        guard.handleSignedData(data, signature);
    }
}
