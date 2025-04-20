// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

interface ITypes {
    struct DataWithId {
        uint64 triggerId;
        bytes data;
    }

    struct TriggerInfo {
        uint64 triggerId;
        address creator;
        bytes data;
    }

    event NewTrigger(bytes);
}
