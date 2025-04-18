// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/contracts/SafeGuard.sol";
import "@gnosis.pm/safe-contracts/contracts/Safe.sol";
import "@gnosis.pm/safe-contracts/contracts/proxies/SafeProxyFactory.sol";
import {Utils} from "./Utils.sol";

// Base contract with shared functionality
contract SafeGuardBaseScript is Script {
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
}

// Deploy contracts script
contract DeploySafeGuardScript is SafeGuardBaseScript {
    function run() public {
        (uint256 deployerPrivateKey, ) = Utils.getPrivateKey(vm);
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

        // Deploy SafeGuard with just the Safe address
        SafeGuard guard = new SafeGuard(payable(safeAddress), serviceManager);
        console.log("Deployed SafeGuard at:", address(guard));

        // Save addresses to .env file
        Utils.saveEnvVars(
            vm,
            string.concat(
                "\nSAFE_ADDRESS=",
                vm.toString(safeAddress),
                "\nGUARD_ADDRESS=",
                vm.toString(address(guard))
            )
        );
        console.log("Saved safe and guard addresses to .env file");

        vm.stopBroadcast();
    }
}

// Create and approve safe transaction script
contract ApproveSafeTransactionScript is SafeGuardBaseScript {
    function _getTxHash(Safe safe) internal view returns (bytes32) {
        // Pack parameters into a struct to reduce stack usage
        return
            safe.getTransactionHash(
                address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266), // to
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
        vm.startBroadcast(ownerPrivateKey);

        address safeAddress = vm.envAddress("SAFE_ADDRESS");
        Safe safe = Safe(payable(safeAddress));

        // Get and approve transaction hash
        bytes32 txHash = _getTxHash(safe);
        safe.approveHash(txHash);
        console.log("Approved transaction hash:", uint256(txHash));

        vm.stopBroadcast();
    }
}

// Execute safe transaction script
contract ExecuteSafeTransactionScript is SafeGuardBaseScript {
    function _getTxHash(Safe safe) internal view returns (bytes32) {
        // Pack parameters into a struct to reduce stack usage
        return
            safe.getTransactionHash(
                address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266), // to
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
        vm.startBroadcast(ownerPrivateKey);

        Safe safe = Safe(payable(vm.envAddress("SAFE_ADDRESS")));

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
            address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266), // to
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
            address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
        );

        vm.stopBroadcast();
    }
}
