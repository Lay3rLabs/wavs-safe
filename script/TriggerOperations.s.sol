// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Script.sol";
import "forge-std/console.sol";
import "../src/contracts/Trigger.sol";
import {Utils} from "./Utils.sol";

contract TriggerBurnShares is Script {
    function run() public {
        // Get deployer private key
        (uint256 deployerPrivateKey, ) = Utils.getPrivateKey(vm);

        // Get trigger contract address from environment
        address triggerAddress = vm.envAddress("TRIGGER_ADDRESS");

        // Get strategy address and shares amount from environment
        address strategy = vm.envAddress("STRATEGY_ADDRESS");
        uint256 shares = vm.envUint("BURN_SHARES_AMOUNT");

        vm.startBroadcast(deployerPrivateKey);

        // Create instance of the Trigger contract
        Trigger trigger = Trigger(triggerAddress);

        // Call triggerBurnShares
        trigger.triggerBurnShares(strategy, shares);

        console.log("Successfully triggered burn shares event");
        console.log("Strategy:", strategy);
        console.log("Shares:", shares);

        vm.stopBroadcast();
    }
}
