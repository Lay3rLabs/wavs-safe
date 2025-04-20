// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {Enum} from "@gnosis.pm/safe-contracts/contracts/common/Enum.sol";
import {ModuleManager} from "@gnosis.pm/safe-contracts/contracts/base/ModuleManager.sol";
import {IWavsServiceHandler} from "@wavs/interfaces/IWavsServiceHandler.sol";
import {IWavsServiceManager} from "@wavs/interfaces/IWavsServiceManager.sol";

contract WavsSafeModule is IWavsServiceHandler {
    // The payload for a transaction to be executed by the Safe, returned from the AVS
    struct TransactionPayload {
        address to;
        uint256 value;
        bytes data;
    }

    // Address of the Gnosis Safe this module is connected to
    address public safe;

    // Store the owner who can use this module
    address public owner;

    // Address of the authorized service manager
    IWavsServiceManager public serviceManager;

    event Funded(address sender, uint256 amount);

    error OnlyOwnerAllowed();
    error OnlySafeAllowed();
    error OnlyServiceManagerAllowed();
    error InvalidSafeAddress();
    error InvalidServiceManagerAddress();
    error InvalidTargetAddress();
    error ModuleTransactionFailed();

    modifier onlyOwner() {
        if (msg.sender != owner) revert OnlyOwnerAllowed();
        _;
    }

    modifier onlySafe() {
        if (msg.sender != safe) revert OnlySafeAllowed();
        _;
    }

    modifier onlyServiceManager() {
        if (msg.sender != address(serviceManager))
            revert OnlyServiceManagerAllowed();
        _;
    }

    constructor(address _safe, address _serviceManager) {
        if (_safe == address(0)) revert InvalidSafeAddress();
        if (_serviceManager == address(0))
            revert InvalidServiceManagerAddress();

        safe = _safe;
        serviceManager = IWavsServiceManager(_serviceManager);
        owner = msg.sender;
    }

    function fundModule() external payable {
        // Accept ETH funding
        emit Funded(msg.sender, msg.value);
    }

    /**
     * @dev Handle the AVS output.
     * @param data The data returned from the oracle AVS.
     * @param signature The signature of the data.
     */
    function handleSignedData(
        bytes calldata data,
        bytes calldata signature
    ) external override {
        serviceManager.validate(data, signature);

        // Decode the transaction from the payload data
        TransactionPayload memory payload = abi.decode(
            data,
            (TransactionPayload)
        );

        if (payload.to == address(0)) revert InvalidTargetAddress();

        // Execute the transaction from the Safe
        bool success = ModuleManager(safe).execTransactionFromModule(
            payload.to,
            payload.value,
            payload.data,
            Enum.Operation.Call
        );

        if (!success) revert ModuleTransactionFailed();
    }
}
