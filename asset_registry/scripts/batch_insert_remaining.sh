#!/bin/bash

# Batch insert script for REMAINING assets from Cosmos Hub chain-registry
# Contract: cosmos1hkcg9jyk48l4va3phemlh2hu6ndtqgfzvl7tle5m4wgfpss08lmq7495zx

CONTRACT="cosmos1hkcg9jyk48l4va3phemlh2hu6ndtqgfzvl7tle5m4wgfpss08lmq7495zx"
ADMIN_KEY="admin"
GAS_FLAGS="--gas auto --gas-adjustment 1.3 -y --fees 1500uatom"

echo "Starting batch insert of remaining Cosmos Hub assets..."

# Check which assets are already added
echo "Checking existing assets..."
EXISTING_ASSETS=$(gaiad query wasm contract-state smart $CONTRACT '{"list_assets":{"limit":50}}' | jq -r '.data[].denom')
echo "Found existing assets: $EXISTING_ASSETS"

# Function to check if asset exists
asset_exists() {
    local denom="$1"
    if echo "$EXISTING_ASSETS" | grep -q "$denom"; then
        return 0  # exists
    else
        return 1  # doesn't exist
    fi
}

# Process remaining assets from assetlist.json
jq -c '.assets[]' assetlist.json | while read -r asset; do
    DENOM=$(echo "$asset" | jq -r '.base')
    NAME=$(echo "$asset" | jq -r '.name')
    SYMBOL=$(echo "$asset" | jq -r '.symbol')
    DESCRIPTION=$(echo "$asset" | jq -r '.description')
    
    # Skip if asset already exists
    if asset_exists "$DENOM"; then
        echo "Skipping $SYMBOL ($DENOM) - already exists"
        continue
    fi
    
    # Get image URL
    IMAGE_URL=$(echo "$asset" | jq -r '.images[0].png // .logo_URIs.png // empty')
    
    # Get decimals (find the denom unit with exponent > 0, default to 6)
    DECIMALS=$(echo "$asset" | jq -r '.denom_units[] | select(.exponent > 0) | .denom' | head -1)
    if [ "$DECIMALS" = "atom" ]; then
        DECIMALS=6
    elif [ "$DECIMALS" = "usdt" ]; then
        DECIMALS=6
    elif [ "$DECIMALS" = "WFX" ]; then
        DECIMALS=18
    elif [ "$DECIMALS" = "crowdp" ]; then
        DECIMALS=18
    elif [ "$DECIMALS" = "wbtc" ]; then
        DECIMALS=8
    elif [ "$DECIMALS" = "lbtc" ]; then
        DECIMALS=8
    elif [ "$DECIMALS" = "usdc" ]; then
        DECIMALS=6
    elif [ "$DECIMALS" = "dai" ]; then
        DECIMALS=18
    elif [ "$DECIMALS" = "link" ]; then
        DECIMALS=18
    elif [ "$DECIMALS" = "uni" ]; then
        DECIMALS=18
    else
        DECIMALS=6  # default
    fi
    
    echo "Adding $SYMBOL ($DENOM)..."
    
    # Build the JSON message
    JSON_MSG=$(cat <<EOF
{
  "add_asset": {
    "denom": "$DENOM",
    "name": "$NAME",
    "ticker": "$SYMBOL",
    "image_url": "$IMAGE_URL",
    "description": "$DESCRIPTION",
    "decimals": $DECIMALS
  }
}
EOF
)
    
    # Execute transaction
    gaiad tx wasm execute $CONTRACT "$JSON_MSG" --from $ADMIN_KEY $GAS_FLAGS
    
    # Wait between transactions
    sleep 6
    
    echo "Completed $SYMBOL"
done

echo "Batch insert completed!"
echo "Verify with: gaiad query wasm contract-state smart $CONTRACT '{\"list_assets\":{\"limit\":50}}'"
