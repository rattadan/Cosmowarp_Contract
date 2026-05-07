# dApp Registry Contract Deployment Guide

## Prerequisites

1. **Docker** - For running the rust-optimizer
2. **Cosmos Hub CLI** (`gaiad`) - For deploying to cosmoshub-4
3. **Wallet with ATOM** - For gas fees

## Step 1: Build Optimized Contract

Run the deployment script to build with rust-optimizer 0.17.0:

```bash
cd dApp_registry
./scripts/deploy.sh
```

This will create an optimized WASM file at `artifacts/dapp_registry.wasm`.

## Step 2: Store Contract Code

Store the contract code on cosmoshub-4:

```bash
TX_HASH=$(gaiad tx wasm store artifacts/dapp_registry.wasm \
  --from <your-key-name> \
  --chain-id cosmoshub-4 \
  --node https://cosmos-rpc.publicnode.com:443 \
  --gas auto --gas-adjustment 1.3 \
  --gas-prices 0.025uatom \
  --broadcast-mode sync \
  --output json -y | jq -r '.txhash')

echo "Transaction Hash: $TX_HASH"
```

## Step 3: Get Code ID

Wait for the transaction to be included (~6 seconds), then query for the code ID:

```bash
sleep 6

CODE_ID=$(gaiad query tx $TX_HASH \
  --node https://cosmos-rpc.publicnode.com:443 \
  --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Code ID: $CODE_ID"
```

## Step 4: Instantiate Contract

Instantiate the contract with your admin address:

```bash
# Replace with your admin address
ADMIN_ADDRESS="cosmos1..."

INIT_MSG='{"admin":"'$ADMIN_ADDRESS'"}'

gaiad tx wasm instantiate $CODE_ID "$INIT_MSG" \
  --from <your-key-name> \
  --label "dApp Registry" \
  --admin $ADMIN_ADDRESS \
  --chain-id cosmoshub-4 \
  --node https://cosmos-rpc.publicnode.com:443 \
  --gas auto --gas-adjustment 1.3 \
  --gas-prices 0.025uatom \
  --broadcast-mode sync \
  --output json -y
```

## Step 5: Get Contract Address

Query for the contract address:

```bash
CONTRACT_ADDRESS=$(gaiad query wasm list-contract-by-code $CODE_ID \
  --node https://cosmos-rpc.publicnode.com:443 \
  --output json | jq -r '.contracts[0]')

echo "Contract Address: $CONTRACT_ADDRESS"
```

## Step 6: Update Frontend Configuration

Update the contract address in your frontend config:

```typescript
// config/dapp-registry.ts
export const DAPP_REGISTRY_CONTRACT_ADDRESS = '<your-contract-address>';
```

## Verify Deployment

Query the contract config to verify:

```bash
gaiad query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"get_config":{}}' \
  --node https://cosmos-rpc.publicnode.com:443 \
  --output json
```

## Example: Add a dApp

```bash
ADD_DAPP_MSG='{
  "add_d_app": {
    "dapp_id": "example-dapp",
    "title": "Example dApp",
    "short_description": "A short description",
    "full_description": "A detailed description of the dApp",
    "logo_url": "https://example.com/logo.png",
    "banner_url": "https://example.com/banner.png",
    "website": "https://example.com",
    "telegram": "https://t.me/example",
    "x": "https://x.com/example",
    "discord": "https://discord.gg/example",
    "github": "https://github.com/example"
  }
}'

gaiad tx wasm execute $CONTRACT_ADDRESS "$ADD_DAPP_MSG" \
  --from <your-key-name> \
  --chain-id cosmoshub-4 \
  --node https://cosmos-rpc.publicnode.com:443 \
  --gas auto --gas-adjustment 1.3 \
  --gas-prices 0.025uatom \
  --broadcast-mode sync \
  --output json -y
```

## Troubleshooting

### Build Issues
- Ensure Docker is running
- Check that you have sufficient disk space
- Try cleaning the build cache: `docker system prune -a`

### Deployment Issues
- Verify you have sufficient ATOM for gas fees
- Check that your wallet is connected to cosmoshub-4
- Ensure the RPC endpoint is accessible

### Query Issues
- Wait a few blocks after transactions
- Try alternative RPC endpoints if one is slow
- Use `--output json` for easier parsing

## Useful Commands

### Query all dApps
```bash
gaiad query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"list_d_apps":{"limit":100}}' \
  --node https://cosmos-rpc.publicnode.com:443
```

### Query user star balance
```bash
gaiad query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"get_user_star_balance":{"user":"cosmos1..."}}' \
  --node https://cosmos-rpc.publicnode.com:443
```

### Distribute stars
```bash
gaiad tx wasm execute $CONTRACT_ADDRESS \
  '{"distribute_stars":{"dapp_id":"example-dapp","stars":5}}' \
  --from <your-key-name> \
  --chain-id cosmoshub-4 \
  --node https://cosmos-rpc.publicnode.com:443 \
  --gas auto --gas-adjustment 1.3 \
  --gas-prices 0.025uatom -y
```
