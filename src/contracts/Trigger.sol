// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {ITypes} from "../interfaces/ITypes.sol";

/**
 * @title Trigger
 * @dev Example contract showing how to create and emit triggers
 */
contract Trigger {
    struct TriggerData {
        address creator;
        bytes data;
    }

    // Address that will receive the payment
    address public recipient;

    // Counter for trigger IDs
    uint64 public nextTriggerId;

    event Funded(address sender, uint256 amount);

    constructor(address _recipient) {
        require(_recipient != address(0), "Invalid recipient address");
        recipient = _recipient;
    }

    /**
     * @dev Add a new trigger with associated data
     * @param data The data to store with the trigger
     * @return triggerId The ID of the newly created trigger
     */
    function addTrigger(
        bytes memory data
    ) external payable returns (uint64 triggerId) {
        require(msg.value == 0.1 ether, "Payment must be exactly 0.1 ETH");

        // Forward the ETH to the recipient using low-level call
        (bool sent, ) = recipient.call{value: msg.value}("");
        require(sent, "ETH transfer failed");

        // Get the next trigger id
        triggerId = nextTriggerId;
        nextTriggerId = nextTriggerId + 1;

        // Create the trigger
        TriggerData memory trigger = TriggerData({
            creator: msg.sender,
            data: data
        });

        // Emit trigger info
        ITypes.TriggerInfo memory triggerInfo = ITypes.TriggerInfo({
            triggerId: triggerId,
            creator: trigger.creator,
            data: trigger.data
        });

        emit ITypes.NewTrigger(abi.encode(triggerInfo));
    }
}
