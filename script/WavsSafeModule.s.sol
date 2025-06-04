// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {Common} from "./Common.s.sol";
import "forge-std/console.sol";
import {stdJson} from "forge-std/StdJson.sol";
import "../src/contracts/WavsSafeModule.sol";
import "../src/contracts/Trigger.sol";
import "../src/contracts/MockUSDC.sol";
import "@gnosis.pm/safe-contracts/contracts/Safe.sol";
import "@gnosis.pm/safe-contracts/contracts/proxies/SafeProxyFactory.sol";
import "@gnosis.pm/safe-contracts/contracts/base/ModuleManager.sol";
import {Strings} from "@openzeppelin-contracts/utils/Strings.sol";
import {Create2} from "@openzeppelin-contracts/utils/Create2.sol";

contract Deploy is Common {
    using stdJson for string;
    using Strings for address;
    using Strings for uint256;

    Safe public safeSingleton;
    SafeProxyFactory public factory;
    address public deployedSafeAddress;
    address public deployedModuleAddress;
    address public deployedTriggerAddress;
    address public deployedUSDCAddress;

    // Salt for deterministic deployment
    bytes32 public constant USDC_SALT = keccak256("MOCK_USDC_SALT_V1");

    // JSON output path
    string public root;
    string public deploymentsPath;

    function run(string calldata serviceManager_) public {
        root = vm.projectRoot();
        deploymentsPath = string.concat(
            root,
            "/.docker/module_deployments.json"
        );

        address serviceManager = vm.parseAddress(serviceManager_);

        vm.startBroadcast(_privateKey);

        // Deploy Safe singleton and factory
        safeSingleton = new Safe();
        factory = new SafeProxyFactory();
        console.log("Deployed Safe singleton at:", address(safeSingleton));
        console.log("Deployed Safe factory at:", address(factory));

        // Deploy Safe if needed
        bool deployNewSafe = vm.envBool("DEPLOY_NEW_SAFE");
        if (deployNewSafe) {
            deployedSafeAddress = _deploySafe(
                _getOwners(),
                vm.envUint("SAFE_THRESHOLD"),
                vm.envAddress("SAFE_FALLBACK_HANDLER")
            );
            console.log("Deployed new Safe at:", deployedSafeAddress);

            // Fund the Safe
            payable(deployedSafeAddress).transfer(1 ether);
            console.log("Funded Safe with 1 ETH");
        } else {
            deployedSafeAddress = vm.envAddress("EXISTING_SAFE_ADDRESS");
            console.log("Using existing Safe at:", deployedSafeAddress);
        }

        // Deploy MockUSDC (or use existing one if already deployed)
        deployedUSDCAddress = _deployMockUSDC();
        console.log("Deployed/Using MockUSDC at:", deployedUSDCAddress);

        // Mint 1 million USDC to the Safe (1,000,000 * 10^6) if it's a fresh deployment
        MockUSDC usdc = MockUSDC(deployedUSDCAddress);
        try usdc.mint(deployedSafeAddress, 1_000_000_000_000) {
            console.log("Minted 1,000,000 USDC to Safe");
        } catch Error(string memory reason) {
            console.log("USDC mint failed:", reason);

            // Try to check the current balance instead
            try usdc.balanceOf(deployedSafeAddress) returns (uint256 balance) {
                console.log("Current Safe USDC balance:", balance / 1e6);
                if (balance == 0) {
                    console.log("Warning: Safe has no USDC balance");
                }
            } catch {
                console.log("Failed to check USDC balance");
            }
        } catch (bytes memory) {
            // Custom error case (likely OwnableUnauthorizedAccount)
            console.log("Failed to mint USDC: Not the owner of the contract");

            // Try to check the current balance instead
            try usdc.balanceOf(deployedSafeAddress) returns (uint256 balance) {
                console.log("Current Safe USDC balance:", balance / 1e6);
                if (balance == 0) {
                    console.log("Warning: Safe has no USDC balance");
                }
            } catch {
                console.log("Failed to check USDC balance");
            }
        }

        // Deploy WavsSafeModule
        WavsSafeModule module = new WavsSafeModule(
            deployedSafeAddress,
            serviceManager
        );
        deployedModuleAddress = address(module);
        console.log("Deployed WavsSafeModule at:", deployedModuleAddress);

        // Deploy Trigger contract (using Safe as recipient)
        Trigger trigger = new Trigger(deployedSafeAddress);
        deployedTriggerAddress = address(trigger);
        console.log("Deployed Trigger contract at:", deployedTriggerAddress);

        // Fund the module
        try module.fundModule{value: 1 ether}() {
            console.log("Funded module with 1 ETH");
        } catch Error(string memory reason) {
            console.log("Failed to fund module:", reason);
            revert(reason);
        }

        // Enable module on Safe if needed
        address safeAddress = module.safe();
        if (vm.envBool("ENABLE_MODULE")) {
            Safe safe = Safe(payable(safeAddress));
            _enableModule(safe, deployedModuleAddress);
            console.log("Enabled module on Safe at:", safeAddress);
        }

        // Write deployment information to JSON file
        writeDeploymentsToJson();

        vm.stopBroadcast();
    }

    function _deploySafe(
        address[] memory owners,
        uint256 threshold,
        address fallbackHandler
    ) internal returns (address) {
        // Use the deployed contracts instead of mainnet addresses
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

    /**
     * @dev Deploys MockUSDC or returns the existing instance
     */
    function _deployMockUSDC() internal returns (address) {
        // Specific address to check first
        address specificAddress = 0xb83251884fe23e61e863a44296f8743c63b291eb;

        // Check if there's already code at this address
        uint256 codeSize;
        assembly {
            codeSize := extcodesize(specificAddress)
        }

        // If a contract exists at the specified address, use it
        if (codeSize > 0) {
            console.log("Found existing MockUSDC at:", specificAddress);
            return specificAddress;
        }

        // Otherwise deploy a new MockUSDC
        MockUSDC usdc = new MockUSDC();
        console.log("Deployed new MockUSDC at:", address(usdc));

        return address(usdc);
    }

    function writeDeploymentsToJson() internal {
        // Create JSON string with deployment information
        string memory json = "{";

        // Add contract addresses
        json = appendJsonPair(
            json,
            "safeSingleton",
            address(safeSingleton),
            true
        );
        json = appendJsonPair(json, "safeFactory", address(factory), false);
        json = appendJsonPair(json, "safeAddress", deployedSafeAddress, false);
        json = appendJsonPair(
            json,
            "wavsSafeModule",
            deployedModuleAddress,
            false
        );
        json = appendJsonPair(
            json,
            "triggerContract",
            deployedTriggerAddress,
            false
        );
        json = appendJsonPair(
            json,
            "serviceHandler",
            deployedModuleAddress,
            false
        );
        json = appendJsonPair(json, "mockUSDC", deployedUSDCAddress, false);

        // Close JSON
        json = string.concat(json, "}");

        // Write JSON to file
        vm.writeFile(deploymentsPath, json);

        console.log("Deployment information saved to:", deploymentsPath);
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

    function _enableModule(Safe safe, address moduleAddress) internal {
        // First check if the Safe exists and has code
        require(address(safe).code.length > 0, "No code at Safe address");

        // Try to get owners to verify it's a valid Safe
        try safe.getOwners() returns (address[] memory owners) {
            require(owners.length > 0, "Safe has no owners");

            bytes memory data = abi.encodeWithSelector(
                ModuleManager.enableModule.selector,
                moduleAddress
            );

            // Execute transaction to enable module
            safe.execTransaction(
                address(safe),
                0,
                data,
                Enum.Operation.Call,
                0,
                0,
                0,
                address(0),
                payable(address(0)),
                _generateSingleSignature(safe)
            );
        } catch {
            revert(
                "Failed to interact with Safe - invalid Safe address or not deployed"
            );
        }
    }

    function _generateSingleSignature(
        Safe safe
    ) internal view returns (bytes memory) {
        // Assumes the deployer is the first owner
        address owner = safe.getOwners()[0];
        return abi.encodePacked(uint256(uint160(owner)), uint256(0), uint8(1));
    }
}

contract AddTrigger is Common {
    using stdJson for string;

    // JSON output path
    string public root;
    string public deploymentsPath;

    function run(string calldata triggerData) public {
        root = vm.projectRoot();
        deploymentsPath = string.concat(
            root,
            "/.docker/module_deployments.json"
        );

        string memory json = vm.readFile(deploymentsPath);
        address triggerAddress = json.readAddress(".triggerContract");

        uint256 balanceBefore = address(
            0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3
        ).balance;

        console.log("Adding trigger to contract at:", triggerAddress);
        console.log("Trigger data:", triggerData);
        console.log("ETH balance before:", balanceBefore);

        Trigger trigger = Trigger(triggerAddress);
        require(address(trigger).code.length > 0, "No code at trigger address");

        vm.startBroadcast(_privateKey);

        // Convert string to bytes
        bytes memory triggerBytes = bytes(triggerData);

        try trigger.addTrigger{value: 0.1 ether}(triggerBytes) {
            console.log("Successfully added trigger");
        } catch Error(string memory reason) {
            console.log("Failed to add trigger:", reason);
            revert(reason);
        }

        vm.stopBroadcast();
    }
}

contract ViewBalance is Common {
    using stdJson for string;

    function run() public view {
        // Get the root directory and deployments path
        string memory root = vm.projectRoot();
        string memory deploymentsPath = string.concat(
            root,
            "/.docker/module_deployments.json"
        );

        // Read deployment information from JSON file
        string memory json = vm.readFile(deploymentsPath);
        address mockUSDCAddress = json.readAddress(".mockUSDC");

        // Address to check balance for
        address targetAddress = 0xDf3679681B87fAE75CE185e4f01d98b64Ddb64a3;

        // Check ETH balance
        uint256 ethBalance = targetAddress.balance;
        console.log("ETH balance:", ethBalance);

        // Check USDC balance if contract is deployed
        if (mockUSDCAddress != address(0)) {
            try MockUSDC(mockUSDCAddress).balanceOf(targetAddress) returns (
                uint256 usdcBalance
            ) {
                // Get decimals to format the balance correctly
                uint8 decimals = MockUSDC(mockUSDCAddress).decimals();
                // Display both raw and formatted balance
                console.log("USDC balance (raw):", usdcBalance);
                console.log(
                    "USDC balance (formatted):",
                    usdcBalance / (10 ** decimals)
                );
            } catch {
                console.log("Failed to check USDC balance");
            }
        } else {
            console.log("MockUSDC address not found in deployments");
        }
    }
}
