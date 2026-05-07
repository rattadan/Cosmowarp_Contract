#!/bin/bash

# Deploy dApp Registry Contract to Cosmos Hub
# Uses rust-optimizer 0.17.0 for optimized builds

set -e

CHAIN_ID="cosmoshub-4"
NODE="https://cosmos-rpc.publicnode.com:443"
ADMIN_ADDRESS="" # Set your admin address here

echo "Building optimized contract with rust-optimizer 0.17.0..."

# Build with docker optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0

echo "Contract built successfully!"
echo "Optimized wasm file: artifacts/dapp_registry.wasm"

# Check if wasm file exists
if [ ! -f "artifacts/dapp_registry.wasm" ]; then
    echo "Error: artifacts/dapp_registry.wasm not found!"
    exit 1
fi

echo ""
echo "To deploy the contract, run:"
echo ""
echo "# Store the contract code"
echo "TX_HASH=\$(osmosisd tx wasm store artifacts/dapp_registry.wasm \\"
echo "  --from <your-key> \\"
echo "  --chain-id $CHAIN_ID \\"
echo "  --node $NODE \\"
echo "  --gas auto --gas-adjustment 1.3 \\"
echo "  --gas-prices 0.025uatom \\"
echo "  --broadcast-mode sync \\"
echo "  --output json -y | jq -r '.txhash')"
echo ""
echo "# Wait for transaction to be included in a block"
echo "sleep 6"
echo ""
echo "# Get the code ID"
echo "CODE_ID=\$(osmosisd query tx \$TX_HASH --node $NODE --output json | jq -r '.logs[0].events[] | select(.type==\"store_code\") | .attributes[] | select(.key==\"code_id\") | .value')"
echo "echo \"Code ID: \$CODE_ID\""
echo ""
echo "# Instantiate the contract"
echo "INIT_MSG='{\"admin\":\"$ADMIN_ADDRESS\"}'"
echo "osmosisd tx wasm instantiate \$CODE_ID \"\$INIT_MSG\" \\"
echo "  --from <your-key> \\"
echo "  --label \"dApp Registry\" \\"
echo "  --admin $ADMIN_ADDRESS \\"
echo "  --chain-id $CHAIN_ID \\"
echo "  --node $NODE \\"
echo "  --gas auto --gas-adjustment 1.3 \\"
echo "  --gas-prices 0.025uatom \\"
echo "  --broadcast-mode sync \\"
echo "  --output json -y"
echo ""
echo "# Query contract address"
echo "osmosisd query wasm list-contract-by-code \$CODE_ID --node $NODE --output json"
