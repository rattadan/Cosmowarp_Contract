# dApp Registry Contract

A CosmWasm smart contract for maintaining an on-chain decentralized application (dApp) registry with a star-based ranking system.

## Overview

The dApp Registry allows developers to register their applications on-chain with comprehensive metadata, social links, and a community-driven star ranking system. Users can distribute stars to their favorite dApps, creating a decentralized reputation system.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    dApp Registry Contract                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   dApp       │  │    Star      │  │    Trace     │      │
│  │  Management  │  │   Ranking    │  │    Data      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                  │               │
│         ▼                 ▼                  ▼               │
│  ┌──────────────────────────────────────────────────┐      │
│  │              Storage Layer                        │      │
│  ├──────────────────────────────────────────────────┤      │
│  │ • DAPPS_BY_ID (dApp entries)                     │      │
│  │ • USER_STAR_BALANCES (user star allocations)     │      │
│  │ • STAR_ASSIGNMENTS (user → dApp star mapping)    │      │
│  │ • CONFIG (admin configuration)                   │      │
│  └──────────────────────────────────────────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Features

- ✅ **dApp Registration**: Anyone can register a dApp with metadata
- ✅ **Star Ranking System**: Users receive 10 stars to distribute to dApps
- ✅ **Star Redelegation**: Users can move stars between dApps
- ✅ **Verification System**: Admin can verify trusted dApps
- ✅ **Trace Data**: Extensible key-value metadata for dApps
- ✅ **Social Links**: Support for website, Twitter/X, Telegram, Discord, GitHub
- ✅ **Admin Controls**: Block/unblock, verify, and remove dApps

## State Structure

### DAppEntry

```rust
pub struct DAppEntry {
    pub dapp_id: String,           // Unique identifier
    pub title: String,             // Display name
    pub short_description: String, // Brief description
    pub full_description: String,  // Detailed description
    pub logo_url: String,          // Logo image URL
    pub banner_url: String,        // Banner image URL
    pub website: String,           // Official website
    pub telegram: Option<String>,  // Telegram link
    pub x: Option<String>,         // Twitter/X link
    pub discord: Option<String>,   // Discord link
    pub github: Option<String>,    // GitHub link
    pub verified: bool,            // Admin verification
    pub blocked: bool,             // Admin block status
    pub created_by: String,        // Creator address
    pub created_at: u64,           // Creation timestamp
    pub updated_at: u64,           // Last update timestamp
    pub trace_data: HashMap<String, String>, // Extensible metadata
    pub total_stars: u32,          // Total stars received
}
```

### Star System

- Each user starts with **10 stars** to distribute
- Stars can be distributed to any dApp
- Stars can be redeemed (taken back) from a dApp
- Stars can be redelegated between dApps
- Total stars per dApp are tracked and displayed

## Execute Messages

### AddDApp

Register a new dApp in the registry.

```json
{
  "add_d_app": {
    "dapp_id": "my-awesome-dapp",
    "title": "My Awesome dApp",
    "short_description": "A revolutionary dApp",
    "full_description": "Detailed description of what makes this dApp special...",
    "logo_url": "https://example.com/logo.png",
    "banner_url": "https://example.com/banner.png",
    "website": "https://myawesomadapp.com",
    "telegram": "https://t.me/myawesomadapp",
    "x": "https://x.com/myawesomadapp",
    "discord": "https://discord.gg/myawesomadapp",
    "github": "https://github.com/myawesomadapp"
  }
}
```

### UpdateDApp

Update an existing dApp (creator or admin only).

```json
{
  "update_d_app": {
    "dapp_id": "my-awesome-dapp",
    "title": "My Awesome dApp v2",
    "short_description": "Updated description",
    "full_description": "Updated full description...",
    "logo_url": "https://example.com/new-logo.png",
    "banner_url": "https://example.com/new-banner.png",
    "website": "https://myawesomadapp.com",
    "telegram": null,
    "x": null,
    "discord": null,
    "github": null
  }
}
```

### DistributeStars

Allocate stars to a dApp (users start with 10 stars).

```json
{
  "distribute_stars": {
    "dapp_id": "my-awesome-dapp",
    "stars": 5
  }
}
```

### RedeemStars

Take back stars from a dApp.

```json
{
  "redeem_stars": {
    "dapp_id": "my-awesome-dapp",
    "stars": 3
  }
}
```

### RedelegateStars

Move stars from one dApp to another.

```json
{
  "redelegate_stars": {
    "from_dapp_id": "old-dapp",
    "to_dapp_id": "new-dapp",
    "stars": 5
  }
}
```

### SetVerified (Admin Only)

Mark a dApp as verified.

```json
{
  "set_verified": {
    "dapp_id": "my-awesome-dapp",
    "verified": true
  }
}
```

### SetBlocked (Admin Only)

Block or unblock a dApp.

```json
{
  "set_blocked": {
    "dapp_id": "spam-dapp",
    "blocked": true
  }
}
```

### RemoveDApp (Admin Only)

Permanently remove a dApp from the registry.

```json
{
  "remove_d_app": {
    "dapp_id": "my-awesome-dapp"
  }
}
```

## Query Messages

### GetDApp

Retrieve a specific dApp by ID.

```json
{
  "get_d_app": {
    "dapp_id": "my-awesome-dapp"
  }
}
```

### ListDApps

List all dApps with pagination and filtering.

```json
{
  "list_d_apps": {
    "start_after": null,
    "limit": 50,
    "include_blocked": false,
    "only_verified": false
  }
}
```

**CLI Example:**
```bash
gaiad query wasm contract-state smart <CONTRACT_ADDRESS> \
  '{"list_d_apps":{"limit":50}}' \
  --node https://rpc.cosmos.network:443
```

**cURL Examples:**
```bash
# List all dApps (default limit: 50)
# Query: {"list_d_apps":{"limit":50}}
# Base64: eyJsaXN0X2RfYXBwcyI6eyJsaW1pdCI6NTB9fQ==
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/<CONTRACT_ADDRESS>/smart/eyJsaXN0X2RfYXBwcyI6eyJsaW1pdCI6NTB9fQ=="

# List only verified dApps
# Query: {"list_d_apps":{"limit":100,"only_verified":true}}
# Base64: eyJsaXN0X2RfYXBwcyI6eyJsaW1pdCI6MTAwLCJvbmx5X3ZlcmlmaWVkIjp0cnVlfX0=
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/<CONTRACT_ADDRESS>/smart/eyJsaXN0X2RfYXBwcyI6eyJsaW1pdCI6MTAwLCJvbmx5X3ZlcmlmaWVkIjp0cnVlfX0="

# List with pagination
# Query: {"list_d_apps":{"start_after":"some-dapp-id","limit":50}}
# Base64: eyJsaXN0X2RfYXBwcyI6eyJzdGFydF9hZnRlciI6InNvbWUtZGFwcC1pZCIsImxpbWl0Ijo1MH19
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/<CONTRACT_ADDRESS>/smart/eyJsaXN0X2RfYXBwcyI6eyJzdGFydF9hZnRlciI6InNvbWUtZGFwcC1pZCIsImxpbWl0Ijo1MH19"

# Helper: Encode your own query
echo -n '{"list_d_apps":{"limit":50}}' | base64
```

### GetUserStarBalance

Get a user's star balance and allocations.

```json
{
  "get_user_star_balance": {
    "user": "cosmos1..."
  }
}
```

### GetDAppsByStars

List dApps sorted by total stars (leaderboard).

```json
{
  "get_d_apps_by_stars": {
    "limit": 100
  }
}
```

### GetConfig

Get contract configuration.

```json
{
  "get_config": {}
}
```

## Deployment Guide

### Prerequisites

1. **Docker** - For running the rust-optimizer
2. **Cosmos Hub CLI** (`gaiad`) - For deploying to cosmoshub-4
3. **Wallet with ATOM** - For gas fees (~0.5 ATOM recommended)

### Step 1: Build Optimized Contract

```bash
cd dApp_registry

# Run the deployment script
./scripts/deploy.sh

# Or use Docker directly
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0
```

This creates `artifacts/dapp_registry.wasm` (~150-200 KB).

### Step 2: Store Contract Code

```bash
# Set variables
WALLET="my-wallet"
CHAIN_ID="cosmoshub-4"
NODE="https://rpc.cosmos.network:443"
GAS_PRICES="0.025uatom"

# Upload wasm
TX_HASH=$(gaiad tx wasm store artifacts/dapp_registry.wasm \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode sync \
  --output json -y | jq -r '.txhash')

echo "Transaction Hash: $TX_HASH"

# Wait for transaction to be included
sleep 6

# Get Code ID
CODE_ID=$(gaiad query tx $TX_HASH \
  --node $NODE \
  --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Code ID: $CODE_ID"
```

### Step 3: Instantiate Contract

```bash
# Set admin address
ADMIN_ADDRESS=$(gaiad keys show $WALLET -a)

# Create instantiate message
INIT_MSG='{"admin":"'$ADMIN_ADDRESS'"}'

# Instantiate contract
gaiad tx wasm instantiate $CODE_ID "$INIT_MSG" \
  --from $WALLET \
  --label "dApp Registry v1" \
  --admin $ADMIN_ADDRESS \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode sync \
  --output json -y

# Get contract address
CONTRACT_ADDRESS=$(gaiad query wasm list-contract-by-code $CODE_ID \
  --node $NODE \
  --output json | jq -r '.contracts[0]')

echo "Contract Address: $CONTRACT_ADDRESS"
```

### Step 4: Verify Deployment

```bash
# Query contract config
gaiad query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"get_config":{}}' \
  --node $NODE \
  --output json
```

## Usage Examples

### Register a dApp

```bash
CONTRACT="<your-contract-address>"

ADD_DAPP_MSG='{
  "add_d_app": {
    "dapp_id": "cosmos-hub-explorer",
    "title": "Cosmos Hub Explorer",
    "short_description": "Explore the Cosmos Hub blockchain",
    "full_description": "A comprehensive blockchain explorer for Cosmos Hub with real-time data, transaction tracking, and validator information.",
    "logo_url": "https://example.com/logo.png",
    "banner_url": "https://example.com/banner.png",
    "website": "https://explorer.cosmos.network",
    "telegram": "https://t.me/cosmosnetwork",
    "x": "https://x.com/cosmos",
    "discord": "https://discord.gg/cosmosnetwork",
    "github": "https://github.com/cosmos/explorer"
  }
}'

gaiad tx wasm execute $CONTRACT "$ADD_DAPP_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

### Distribute Stars to a dApp

```bash
DISTRIBUTE_MSG='{
  "distribute_stars": {
    "dapp_id": "cosmos-hub-explorer",
    "stars": 5
  }
}'

gaiad tx wasm execute $CONTRACT "$DISTRIBUTE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

### Query dApp Leaderboard

```bash
gaiad query wasm contract-state smart $CONTRACT \
  '{"get_d_apps_by_stars":{"limit":10}}' \
  --node $NODE
```

### Check Your Star Balance

```bash
USER_ADDRESS=$(gaiad keys show $WALLET -a)

gaiad query wasm contract-state smart $CONTRACT \
  '{"get_user_star_balance":{"user":"'$USER_ADDRESS'"}}' \
  --node $NODE
```

## Star System Flow

```
┌──────────────────────────────────────────────────────────┐
│                    User Joins                             │
│              (Receives 10 stars)                          │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────────┐
│              Distribute Stars                             │
│         (Allocate to favorite dApps)                      │
└────────────────────┬─────────────────────────────────────┘
                     │
                     ├─────────────────┬────────────────────┐
                     ▼                 ▼                    ▼
         ┌──────────────────┐ ┌──────────────┐  ┌──────────────┐
         │  Redeem Stars    │ │  Redelegate  │  │  Keep Stars  │
         │  (Take back)     │ │  (Move to    │  │  (Maintain   │
         │                  │ │   another)   │  │   support)   │
         └──────────────────┘ └──────────────┘  └──────────────┘
```

## Admin Operations

### Verify a dApp

```bash
gaiad tx wasm execute $CONTRACT \
  '{"set_verified":{"dapp_id":"cosmos-hub-explorer","verified":true}}' \
  --from admin_key \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

### Block a Spam dApp

```bash
gaiad tx wasm execute $CONTRACT \
  '{"set_blocked":{"dapp_id":"spam-dapp","blocked":true}}' \
  --from admin_key \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

## Troubleshooting

### "dApp ID already exists"
- The dApp ID must be unique. Choose a different ID or query the existing dApp.

### "Insufficient stars"
- You only have 10 stars total. Redeem stars from other dApps first.

### "Unauthorized"
- Only the dApp creator or admin can update/remove dApps.

### Gas Issues
```bash
# Increase gas limit
--gas 2000000

# Or increase gas adjustment
--gas-adjustment 1.5
```

## Integration with Frontend

### TypeScript Example

```typescript
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

// Add a dApp
const addDAppMsg = {
  add_d_app: {
    dapp_id: "my-dapp",
    title: "My dApp",
    short_description: "Brief description",
    full_description: "Detailed description",
    logo_url: "https://...",
    banner_url: "https://...",
    website: "https://...",
    telegram: null,
    x: null,
    discord: null,
    github: null
  }
};

await client.execute(
  senderAddress,
  contractAddress,
  addDAppMsg,
  "auto"
);

// Query dApps
const dApps = await client.queryContractSmart(
  contractAddress,
  { list_d_apps: { limit: 100 } }
);
```

## Security Considerations

1. **Admin Key Security**: Store admin private key securely
2. **Star Manipulation**: Users cannot create stars, only redistribute their initial 10
3. **dApp Verification**: Only admin can verify dApps
4. **Rate Limiting**: Consider implementing rate limits in frontend

## Future Enhancements

- [ ] Category/tag system for dApps
- [ ] User reviews and comments
- [ ] dApp analytics tracking
- [ ] Featured dApps section
- [ ] Search functionality
- [ ] Multi-chain support

## License

MIT
