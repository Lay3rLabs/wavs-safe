// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {ITypes} from "../interfaces/ITypes.sol";
import {IStrategyManager} from "../interfaces/IStrategyManager.sol";
/**
 * @title Trigger
 * @dev Example contract showing how to create and emit triggers
 */
contract Trigger is IStrategyManager {
    struct TriggerData {
        address creator;
        bytes data;
    }

    // Address that will receive the payment
    address public recipient;

    // Counter for trigger IDs
    uint64 public nextTriggerId;

    // Counter for total shares burned
    uint256 public totalSharesBurned;

    // Mapping to track shares burned per strategy
    mapping(address => uint256) public sharesBurnedByStrategy;

    event Funded(address sender, uint256 amount);
    event SharesBurned(address strategy, uint256 amount);

    error InvalidRecipientAddress();
    error PaymentAmountIncorrect();
    error EthTransferFailed();

    constructor(address _recipient) {
        if (_recipient == address(0)) revert InvalidRecipientAddress();
        recipient = _recipient;
    }

    // For Eigen demo
    event BurnableSharesIncreased(address strategy, uint256 shares);

    /**
     * @dev Triggers the BurnableSharesIncreased event
     * @param strategy The address of the strategy
     * @param shares The number of shares to increase
     */
    function triggerBurnShares(address strategy, uint256 shares) external {
        emit BurnableSharesIncreased(strategy, shares);
    }

    /**
     * @dev Burns shares for the specified strategy
     * @param strategy The address of the strategy
     */
    function burnShares(address strategy) external override {
        // For this demo, we'll just increment counters (1 share per burn)
        totalSharesBurned++;
        sharesBurnedByStrategy[strategy]++;

        emit SharesBurned(strategy, 1);
    }

    /**
     * @dev Returns the number of shares burned for a specific strategy
     * @param strategy The address of the strategy
     * @return The number of shares burned
     */
    function getSharesBurned(address strategy) external view returns (uint256) {
        return sharesBurnedByStrategy[strategy];
    }

    /**
     * @dev Add a new trigger with associated data
     * @param data The data to store with the trigger
     * @return triggerId The ID of the newly created trigger
     */
    function addTrigger(
        bytes memory data
    ) external payable returns (uint64 triggerId) {
        if (msg.value != 0.1 ether) revert PaymentAmountIncorrect();

        // Forward the ETH to the recipient using low-level call
        (bool sent, ) = recipient.call{value: msg.value}("");
        if (!sent) revert EthTransferFailed();

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
