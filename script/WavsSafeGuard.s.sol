// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";
import "../src/contracts/WavsSafeGuard.sol";
import "@gnosis.pm/safe-contracts/contracts/Safe.sol";
import "@gnosis.pm/safe-contracts/contracts/proxies/SafeProxyFactory.sol";
import {Utils} from "./Utils.sol";
import {Strings} from "@openzeppelin-contracts/utils/Strings.sol";

// Base contract with shared functionality
contract WavsSafeGuardBase is Script {
    using stdJson for string;
    using Strings for address;
    using Strings for uint256;

    // Guard storage slot from GuardManager.sol
    // keccak256("guard_manager.guard.address")
    bytes32 internal constant GUARD_STORAGE_SLOT =
        0x4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c8;

    // JSON output path
    string public root;
    string public deploymentsPath;

    Safe public safeSingleton;
    SafeProxyFactory public factory;

    function _deploySafe(
        address[] memory owners,
        uint256 threshold,
        address fallbackHandler
    ) internal returns (address) {
        bytes memory initializer = abi.encodeWithSelector(
            Safe.setup.selector,
            owners,
            threshold,
            address(0),
            "",
            fallbackHandler,
            address(0),
            0,
            payable(address(0))
        );

        address safeAddress = address(
            factory.createProxyWithNonce(address(safeSingleton), initializer, 0)
        );

        return safeAddress;
    }

    function _getOwners() internal view returns (address[] memory) {
        string memory ownersRaw = vm.envString("SAFE_OWNERS");
        string[] memory ownerStrings = _split(ownersRaw, ",");

        address[] memory owners = new address[](ownerStrings.length);
        for (uint i = 0; i < ownerStrings.length; i++) {
            owners[i] = vm.parseAddress(ownerStrings[i]);
        }
        return owners;
    }

    function _loadDeployments()
        internal
        returns (address safeAddress, address guardAddress)
    {
        root = vm.projectRoot();
        deploymentsPath = string.concat(
            root,
            "/.docker/guard_deployments.json"
        );

        // Check if file exists
        try vm.readFile(deploymentsPath) returns (string memory content) {
            string memory json = content;
            safeAddress = json.readAddress(".safeAddress");
            guardAddress = json.readAddress(".guardAddress");
        } catch {
            // File doesn't exist or couldn't be read
            safeAddress = address(0);
            guardAddress = address(0);
        }
    }

    function appendJsonPair(
        string memory _json,
        string memory _key,
        address _value,
        bool _isFirst
    ) internal pure returns (string memory) {
        string memory prefix = _isFirst ? "" : ",";
        return
            string.concat(
                _json,
                prefix,
                '"',
                _key,
                '":"',
                Strings.toHexString(_value),
                '"'
            );
    }

    // Helper functions for string manipulation
    function _split(
        string memory _str,
        string memory _delimiter
    ) internal pure returns (string[] memory) {
        uint count = 1;
        for (uint i = 0; i < bytes(_str).length; i++) {
            if (bytes(_str)[i] == bytes(_delimiter)[0]) count++;
        }

        string[] memory parts = new string[](count);
        count = 0;

        uint lastIndex = 0;
        for (uint i = 0; i < bytes(_str).length; i++) {
            if (bytes(_str)[i] == bytes(_delimiter)[0]) {
                parts[count] = _substring(_str, lastIndex, i);
                lastIndex = i + 1;
                count++;
            }
        }
        parts[count] = _substring(_str, lastIndex, bytes(_str).length);

        return parts;
    }

    function _substring(
        string memory _str,
        uint _start,
        uint _end
    ) internal pure returns (string memory) {
        bytes memory strBytes = bytes(_str);
        bytes memory result = new bytes(_end - _start);
        for (uint i = _start; i < _end; i++) {
            result[i - _start] = strBytes[i];
        }
        return string(result);
    }

    function _writeDeploymentsToJson(
        address safeAddress,
        address guardAddress,
        address singleton,
        address safeFactory
    ) internal {
        // Create JSON string with deployment information
        string memory json = "{";

        // Add contract addresses
        json = appendJsonPair(json, "safeSingleton", singleton, true);
        json = appendJsonPair(json, "safeFactory", safeFactory, false);
        json = appendJsonPair(json, "safeAddress", safeAddress, false);
        json = appendJsonPair(json, "guardAddress", guardAddress, false);

        // Close JSON
        json = string.concat(json, "}");

        // Create directories if they don't exist
        string memory dirPath = string.concat(root, "/deployments");
        vm.createDir(dirPath, true);

        // Write JSON to file
        vm.writeFile(deploymentsPath, json);

        console.log("Deployment information saved to:", deploymentsPath);
    }

    // Get the guard address from storage
    function _getGuardFromStorage(
        address safeAddress
    ) internal view returns (address guard) {
        bytes32 slot = GUARD_STORAGE_SLOT;
        bytes32 value = vm.load(safeAddress, slot);
        guard = address(uint160(uint256(value)));
    }
}

// Deploy contracts script
contract Deploy is WavsSafeGuardBase {
    function run() public {
        (uint256 deployerPrivateKey, ) = Utils.getPrivateKey(vm);

        // Initialize deployment paths
        root = vm.projectRoot();
        deploymentsPath = string.concat(
            root,
            "/.docker/guard_deployments.json"
        );

        vm.startBroadcast(deployerPrivateKey);

        // Deploy Safe singleton and factory first if needed
        safeSingleton = new Safe();
        factory = new SafeProxyFactory();
        console.log("Deployed Safe singleton at:", address(safeSingleton));
        console.log("Deployed Safe factory at:", address(factory));

        // Get Safe setup parameters from environment
        address[] memory owners = _getOwners();
        uint256 threshold = vm.envUint("SAFE_THRESHOLD");
        address fallbackHandler = vm.envAddress("SAFE_FALLBACK_HANDLER");

        // Deploy new Safe if DEPLOY_NEW_SAFE is true
        address safeAddress;
        if (vm.envBool("DEPLOY_NEW_SAFE")) {
            safeAddress = _deploySafe(owners, threshold, fallbackHandler);
            console.log("Deployed new Safe at:", safeAddress);
        } else {
            safeAddress = vm.envAddress("EXISTING_SAFE_ADDRESS");
            console.log("Using existing Safe at:", safeAddress);
        }

        address serviceManager = Utils.getServiceManager(vm);

        // Deploy WavsSafeGuard with just the Safe address
        WavsSafeGuard guard = new WavsSafeGuard(
            payable(safeAddress),
            serviceManager
        );
        console.log("Deployed WavsSafeGuard at:", address(guard));

        // Write deployment information to JSON before trying to enable the guard
        // to ensure we save the addresses even if the enabling fails
        _writeDeploymentsToJson(
            safeAddress,
            address(guard),
            address(safeSingleton),
            address(factory)
        );

        // Now try to enable the guard
        _enableGuard(safeAddress, address(guard), deployerPrivateKey);

        vm.stopBroadcast();
    }

    // Separate function to enable guard to avoid stack too deep errors
    function _enableGuard(
        address safeAddress,
        address guardAddress,
        uint256 signerKey
    ) internal {
        Safe safe = Safe(payable(safeAddress));
        uint256 safeThreshold = safe.getThreshold();

        // Create setGuard transaction data
        bytes memory setGuardData = abi.encodeWithSignature(
            "setGuard(address)",
            guardAddress
        );

        // Calculate transaction hash
        bytes32 txHash = safe.getTransactionHash(
            safeAddress, // to: the Safe itself
            0, // value
            setGuardData, // data: call setGuard
            Enum.Operation.Call, // operation
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            safe.nonce() // nonce
        );

        // Sign the transaction
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(signerKey, txHash);
        bytes memory signature = abi.encodePacked(r, s, v);

        // If threshold > 1, approve the transaction hash
        if (safeThreshold > 1) {
            safe.approveHash(txHash);
            console.log("Transaction approved by deployer");
            console.log("Safe threshold is", safeThreshold);
            console.log("Additional approvals may be needed from other owners");
            console.log(
                "Transaction hash for other owners to approve:",
                vm.toString(txHash)
            );
        }

        // Try to execute the transaction
        try
            safe.execTransaction(
                safeAddress, // to: the Safe itself
                0, // value
                setGuardData, // data: call setGuard
                Enum.Operation.Call, // operation
                0, // safeTxGas
                0, // baseGas
                0, // gasPrice
                address(0), // gasToken
                payable(address(0)), // refundReceiver
                signature // signature
            )
        {
            console.log("Guard successfully enabled on Safe");

            // Verify the guard is set by reading the storage directly
            address setGuard = _getGuardFromStorage(safeAddress);
            if (setGuard == guardAddress) {
                console.log("Verified: Guard is properly set");
            } else {
                console.log(
                    "Error: Guard is set to a different address:",
                    setGuard
                );
            }
        } catch Error(string memory reason) {
            console.log("Failed to enable guard:", reason);
            if (safeThreshold > 1) {
                console.log(
                    "If this is due to threshold > 1, other owners need to approve hash:",
                    vm.toString(txHash)
                );
                console.log(
                    "Run the EnableGuard script after all owners have approved"
                );
            }
        } catch {
            console.log("Failed to enable guard with unknown error");
            console.log(
                "If this is due to threshold > 1, other owners need to approve hash:",
                vm.toString(txHash)
            );
        }
    }
}

// Create and approve safe transaction script
contract ApproveSafeTransaction is WavsSafeGuardBase {
    function _getTxHash(Safe safe) internal view returns (bytes32) {
        // Pack parameters into a struct to reduce stack usage
        return
            safe.getTransactionHash(
                address(0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3), // to
                0.1 ether, // value
                "", // data
                Enum.Operation.Call, // operation
                0, // safeTxGas
                0, // baseGas
                0, // gasPrice
                address(0), // gasToken
                payable(address(0)), // refundReceiver
                safe.nonce() // nonce
            );
    }

    function run() public {
        (uint256 ownerPrivateKey, ) = Utils.getPrivateKey(vm);

        // Load deployments from JSON
        (address safeAddress, ) = _loadDeployments();
        require(
            safeAddress != address(0),
            "Safe address not found in deployments"
        );

        vm.startBroadcast(ownerPrivateKey);

        Safe safe = Safe(payable(safeAddress));

        // Get and approve transaction hash
        bytes32 txHash = _getTxHash(safe);
        safe.approveHash(txHash);
        console.log("Approved transaction hash:", uint256(txHash));

        vm.stopBroadcast();
    }
}

// Execute safe transaction script
contract ExecuteSafeTransaction is WavsSafeGuardBase {
    function _getTxHash(Safe safe) internal view returns (bytes32) {
        // Pack parameters into a struct to reduce stack usage
        return
            safe.getTransactionHash(
                address(0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3), // to
                0.1 ether, // value
                "", // data
                Enum.Operation.Call, // operation
                0, // safeTxGas
                0, // baseGas
                0, // gasPrice
                address(0), // gasToken
                payable(address(0)), // refundReceiver
                safe.nonce() // nonce
            );
    }

    function run() public {
        (uint256 ownerPrivateKey, ) = Utils.getPrivateKey(vm);

        // Load deployments from JSON
        (address safeAddress, ) = _loadDeployments();
        require(
            safeAddress != address(0),
            "Safe address not found in deployments"
        );

        vm.startBroadcast(ownerPrivateKey);

        Safe safe = Safe(payable(safeAddress));

        // First, fund the Safe with more than needed ETH
        (bool success, ) = address(safe).call{value: 0.2 ether}("");
        require(success, "Failed to send ETH to Safe");
        console.log("Funded Safe with 0.2 ETH");

        // Sign and execute in one step
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(
            ownerPrivateKey,
            _getTxHash(safe)
        );
        safe.execTransaction(
            address(0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3), // to
            0.1 ether, // value
            "", // data
            Enum.Operation.Call, // operation
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            abi.encodePacked(r, s, v) // signature
        );

        console.log(
            "Executed transaction to:",
            address(0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3)
        );

        vm.stopBroadcast();
    }
}

// Enable Guard on an existing Safe
contract EnableGuard is WavsSafeGuardBase {
    function run() public {
        (uint256 ownerPrivateKey, ) = Utils.getPrivateKey(vm);

        // Load deployments from JSON
        (address safeAddress, address guardAddress) = _loadDeployments();
        require(
            safeAddress != address(0),
            "Safe address not found in deployments"
        );
        require(
            guardAddress != address(0),
            "Guard address not found in deployments"
        );

        console.log("Safe address:", safeAddress);
        console.log("Guard address:", guardAddress);

        vm.startBroadcast(ownerPrivateKey);

        Safe safe = Safe(payable(safeAddress));
        uint256 threshold = safe.getThreshold();
        console.log("Safe threshold:", threshold);

        // Create setGuard transaction data
        bytes memory setGuardData = abi.encodeWithSignature(
            "setGuard(address)",
            guardAddress
        );

        // Calculate transaction hash
        bytes32 txHash = safe.getTransactionHash(
            safeAddress, // to: the Safe itself
            0, // value
            setGuardData, // data: call setGuard
            Enum.Operation.Call, // operation
            0, // safeTxGas
            0, // baseGas
            0, // gasPrice
            address(0), // gasToken
            payable(address(0)), // refundReceiver
            safe.nonce() // nonce
        );

        // Sign the transaction
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(ownerPrivateKey, txHash);
        bytes memory signature = abi.encodePacked(r, s, v);

        // If threshold > 1, approve the transaction hash
        if (threshold > 1) {
            safe.approveHash(txHash);
            console.log("Transaction approved by current signer");
            console.log("If threshold > 1, additional approvals needed");
            console.log(
                "Transaction hash for other signers to approve:",
                vm.toString(txHash)
            );
        }

        // Try to execute the transaction
        try
            safe.execTransaction(
                safeAddress, // to: the Safe itself
                0, // value
                setGuardData, // data: call setGuard
                Enum.Operation.Call, // operation
                0, // safeTxGas
                0, // baseGas
                0, // gasPrice
                address(0), // gasToken
                payable(address(0)), // refundReceiver
                signature // signature
            )
        {
            console.log("Guard successfully enabled on Safe");

            // Verify the guard is set by reading the storage directly
            address setGuard = _getGuardFromStorage(safeAddress);
            if (setGuard == guardAddress) {
                console.log("Verified: Guard is properly set");
            } else {
                console.log(
                    "Error: Guard is set to a different address:",
                    setGuard
                );
            }
        } catch Error(string memory reason) {
            console.log("Failed to enable guard:", reason);
            console.log(
                "If this is due to threshold > 1, other owners need to approve hash:",
                vm.toString(txHash)
            );
        } catch {
            console.log("Failed to enable guard with unknown error");
            console.log(
                "If this is due to threshold > 1, other owners need to approve hash:",
                vm.toString(txHash)
            );
        }

        vm.stopBroadcast();
    }
}
