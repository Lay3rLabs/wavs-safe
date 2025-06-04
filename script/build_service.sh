#!/bin/bash

set -e
# set -x

: '''
# Run:

sh ./build_service.sh

# Overrides:
- FILE_LOCATION: The save location of the configuration file
- FUEL_LIMIT: The fuel limit (wasm compute metering) for the service
- MAX_GAS: The maximum chain gas for the submission Tx
'''

# == Defaults ==

FUEL_LIMIT=${FUEL_LIMIT:-1000000000000}
MAX_GAS=${MAX_GAS:-5000000}
FILE_LOCATION=${FILE_LOCATION:-".docker/service.json"}
TRIGGER_CHAIN=${TRIGGER_CHAIN:-"local"}
SUBMIT_CHAIN=${SUBMIT_CHAIN:-"local"}
AGGREGATOR_URL=${AGGREGATOR_URL:-""}
DEPLOY_ENV=${DEPLOY_ENV:-""}
# used in make upload-component
WAVS_ENDPOINT=${WAVS_ENDPOINT:-"http://localhost:8000"}
REGISTRY=${REGISTRY:-"wa.dev"}

# === Safe Module DAO Agent ===
SAFE_MODULE_TRIGGER_ADDR=`jq -r '.triggerContract' .docker/module_deployments.json 2>/dev/null || echo ""`
SAFE_MODULE_SUBMIT_ADDR=`jq -r '.wavsSafeModule' .docker/module_deployments.json 2>/dev/null || echo ""`
SAFE_MODULE_TRIGGER_EVENT="NewTrigger(bytes)"
SAFE_MODULE_PKG_NAME="dao-agent"
SAFE_MODULE_ENV_VARS="WAVS_ENV_OPENAI_API_KEY,WAVS_ENV_OPENAI_API_URL,WAVS_ENV_IPFS_GATEWAY_URL"

# === Safe Guard ===
SAFE_GUARD_TRIGGER_ADDR=`jq -r '.safeAddress' .docker/guard_deployments.json 2>/dev/null || echo ""`
SAFE_GUARD_SUBMIT_ADDR=`jq -r '.guardAddress' .docker/guard_deployments.json 2>/dev/null || echo ""`
SAFE_GUARD_TRIGGER_EVENT="ApproveHash(bytes32,address)"
SAFE_GUARD_PKG_NAME="safe-guard"
SAFE_GUARD_ENV_VARS="WAVS_ENV_OPENAI_API_KEY,WAVS_ENV_OPENAI_API_URL,WAVS_ENV_IPFS_GATEWAY_URL"

BASE_CMD="docker run --rm --network host -w /data -v $(pwd):/data ghcr.io/lay3rlabs/wavs:0.4.0-rc wavs-cli service --json true --home /data --file /data/${FILE_LOCATION}"

if [ -z "$WAVS_SERVICE_MANAGER_ADDRESS" ]; then
    # DevEx: attempt to grab it from the location if not set already
    export WAVS_SERVICE_MANAGER_ADDRESS=$(jq -r .addresses.WavsServiceManager ./.nodes/avs_deploy.json)

    if [ -z "$WAVS_SERVICE_MANAGER_ADDRESS" ]; then
        echo "WAVS_SERVICE_MANAGER_ADDRESS is not set. Please set it to the address of the service manager."
        exit 1
    fi
fi

if [ -z "$DEPLOY_ENV" ]; then
    DEPLOY_ENV=$(sh ./script/get-deploy-status.sh)
fi
# === Core ===

SERVICE_ID=`$BASE_CMD init --name demo | jq -r .service.id`
echo "Service ID: ${SERVICE_ID}"

function new_workflow() {
    local trigger_address=$1
    local submit_address=$2
    local event_type=$3 # "event" or "cron"
    local trigger_event_or_cron_schedule=$4
    local pkg_name=$5
    local env_vars=$6
    local config=$7

    local workflow_id=`$BASE_CMD workflow add | jq -r .workflow_id`
    echo "Workflow ID: ${workflow_id}"

    if [ "${event_type}" == "event" ]; then
        local trigger_event_hash=`cast keccak ${trigger_event_or_cron_schedule}`
        $BASE_CMD workflow trigger --id ${workflow_id} set-evm --address ${trigger_address} --chain-name ${TRIGGER_CHAIN} --event-hash ${trigger_event_hash} > /dev/null
    elif [ "${event_type}" == "cron" ]; then
        # no CMD for cron yet, edit the service.json file directly
        tmp=$(mktemp)
        jq '.workflows["'${workflow_id}'"].trigger = { "cron": { "schedule": "'"$trigger_event_or_cron_schedule"'", "start_time": null, "end_time": null } }' ${FILE_LOCATION} > ${tmp}
        mv ${tmp} ${FILE_LOCATION}
    fi

    # If no aggregator is set, use the default (during workflow submit)
    WORKFLOW_SUB_CMD="set-evm"
    if [ -n "$AGGREGATOR_URL" ]; then
        WORKFLOW_SUB_CMD="set-aggregator --url ${AGGREGATOR_URL}"
    fi
    $BASE_CMD workflow submit --id ${workflow_id} ${WORKFLOW_SUB_CMD} --address ${submit_address} --chain-name ${SUBMIT_CHAIN} --max-gas ${MAX_GAS} > /dev/null

    $BASE_CMD workflow component --id ${workflow_id} set-source-registry --domain ${REGISTRY} --package ${PKG_NAMESPACE}:${pkg_name} --version ${PKG_VERSION}

    $BASE_CMD workflow component --id ${workflow_id} permissions --http-hosts '*' --file-system true > /dev/null
    $BASE_CMD workflow component --id ${workflow_id} time-limit --seconds 60 > /dev/null
    if [ -n "${env_vars}" ]; then
        $BASE_CMD workflow component --id ${workflow_id} env --values ${env_vars} > /dev/null
    fi
    if [ -n "${config}" ]; then
        $BASE_CMD workflow component --id ${workflow_id} config --values ${config} > /dev/null
    fi
}

if [[ "${DEMO}" == "module" ]]; then
    new_workflow ${SAFE_MODULE_TRIGGER_ADDR} ${SAFE_MODULE_SUBMIT_ADDR} "event" ${SAFE_MODULE_TRIGGER_EVENT} ${SAFE_MODULE_PKG_NAME} ${SAFE_MODULE_ENV_VARS} ""
elif [[ "${DEMO}" == "guard" ]]; then
    new_workflow ${SAFE_GUARD_TRIGGER_ADDR} ${SAFE_GUARD_SUBMIT_ADDR} "event" ${SAFE_GUARD_TRIGGER_EVENT} ${SAFE_GUARD_PKG_NAME} ${SAFE_GUARD_ENV_VARS} ""
else
    echo "Invalid DEMO value: ${DEMO}. Expected 'module' or 'guard'."
    exit 1
fi

$BASE_CMD manager set-evm --chain-name ${SUBMIT_CHAIN} --address `cast --to-checksum ${WAVS_SERVICE_MANAGER_ADDRESS}` > /dev/null
$BASE_CMD validate > /dev/null

echo "Configuration file created ${FILE_LOCATION}. Watching events from '${TRIGGER_CHAIN}' & submitting to '${SUBMIT_CHAIN}'."
