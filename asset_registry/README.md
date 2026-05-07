# Asset Registry Contract

A CosmWasm smart contract for maintaining an on-chain asset list repository.

**Deployed Contract Address**: `cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5` (Cosmos Hub)

juno16fkpf345shzjkc0yv686vsvcy0eeefqed3sjmpgjftldhyj0ju6skly03a   on juno


Anyone can add a new asset entry as long as both:
- the `denom` is not already registered, and
- the `ticker` is not already registered (case-insensitive; stored uppercase)

Only the asset creator or the contract admin can update an entry's mutable fields.
Only the contract admin can update the ticker field, set `verified`, set `blocked`, and remove assets.

## Stored fields

Each entry stores:
- denom
- name
- ticker
- image_url
- description (legacy field, maintained for backward compatibility)
- structured_descriptions (key-value map for extensible fields)
- website
- x
- discord
- telegram
- decimals (defaults to 6)
- verified (admin-only)
- blocked (admin-only)
- created_by
- created_at
- updated_at

## Messages

### Instantiate

```json
{ "admin": "cosmos1..." }
```

### Execute

#### AddAsset

```json
{
  "add_asset": {
    "denom": "ibc/...",
    "name": "Ethereum",
    "ticker": "ETH",
    "image_url": "https://.../eth.svg",
    "description": "Ethereum",
    "website": "https://ethereum.org",
    "x": "https://x.com/ethereum",
    "discord": "https://discord.gg/ethereum",
    "telegram": "https://t.me/ethereum",
    "decimals": 18
  }
}
```

If `decimals` is omitted, the contract assumes `6`.

#### UpdateAsset (creator or admin only)

```json
{
  "update_asset": {
    "denom": "ibc/...",
    "name": "Ethereum",
    "ticker": "ETH2",
    "image_url": "https://.../eth.svg",
    "description": "Ethereum updated",
    "website": "https://ethereum.org",
    "x": "https://x.com/ethereum",
    "discord": "https://discord.gg/ethereum",
    "telegram": "https://t.me/ethereum",
    "decimals": 18
  }
}
```

Note: Only the contract admin can update the `ticker` field. All other fields can be updated by either the asset creator or the admin.

#### SetVerified (admin only)

```json
{ "set_verified": { "denom": "ibc/...", "verified": true } }
```

#### SetBlocked (admin only)

```json
{ "set_blocked": { "denom": "ibc/...", "blocked": true } }
```

#### UpdateConfig (admin only)

```json
{ "update_config": { "admin": "cosmos1..." } }
```

#### RemoveAsset (admin only)

```json
{ "remove_asset": { "denom": "ibc/..." } }
```

#### RemoveStructuredDescription (creator or admin only)

```json
{ "remove_structured_description": { "denom": "ibc/...", "key": "technical_details" } }
```

## Queries

### GetAssetByDenom

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_asset_by_denom":{"denom":"ibc/..."}}'
```

### GetAssetByTicker

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_asset_by_ticker":{"ticker":"ETH"}}'
```

### ListAssets

**CLI Examples:**
```bash
# List all assets (paginated)
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"list_assets":{"limit":10}}'

# List only verified assets
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"list_assets":{"only_verified":true}}'

# List assets excluding blocked ones
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"list_assets":{"include_blocked":false}}'

# Pagination with start_after
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"list_assets":{"start_after":"ibc/...","limit":10}}'
```

**cURL Examples:**
```bash
# List all registered tokens (default limit: 100)
# Query: {"list_assets":{"limit":100}}
# Base64: eyJsaXN0X2Fzc2V0cyI6eyJsaW1pdCI6MTAwfX0=
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5/smart/eyJsaXN0X2Fzc2V0cyI6eyJsaW1pdCI6MTAwfX0="

# List only verified tokens
# Query: {"list_assets":{"only_verified":true,"limit":100}}
# Base64: eyJsaXN0X2Fzc2V0cyI6eyJvbmx5X3ZlcmlmaWVkIjp0cnVlLCJsaW1pdCI6MTAwfX0=
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5/smart/eyJsaXN0X2Fzc2V0cyI6eyJvbmx5X3ZlcmlmaWVkIjp0cnVlLCJsaW1pdCI6MTAwfX0="

# List tokens excluding blocked ones
# Query: {"list_assets":{"include_blocked":false,"limit":100}}
# Base64: eyJsaXN0X2Fzc2V0cyI6eyJpbmNsdWRlX2Jsb2NrZWQiOmZhbHNlLCJsaW1pdCI6MTAwfX0=
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5/smart/eyJsaXN0X2Fzc2V0cyI6eyJpbmNsdWRlX2Jsb2NrZWQiOmZhbHNlLCJsaW1pdCI6MTAwfX0="

# Pagination with start_after
# Query: {"list_assets":{"start_after":"uatom","limit":50}}
# Base64: eyJsaXN0X2Fzc2V0cyI6eyJzdGFydF9hZnRlciI6InVhdG9tIiwibGltaXQiOjUwfX0=
curl -X GET \
  "https://cosmos-rest.publicnode.com/cosmwasm/wasm/v1/contract/cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5/smart/eyJsaXN0X2Fzc2V0cyI6eyJzdGFydF9hZnRlciI6InVhdG9tIiwibGltaXQiOjUwfX0="

# Helper: Encode your own query
echo -n '{"list_assets":{"limit":100}}' | base64
```

### GetConfig

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_config":{}}'
```

### GetStructuredDescription

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_structured_description":{"denom":"ibc/...","key":"technical_details"}}'
```

### GetAllStructuredDescriptions

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_all_structured_descriptions":{"denom":"ibc/..."}}'
```

## Tutorial: Adding an Asset Entry

This comprehensive tutorial shows how to add a new asset to the registry.

### Prerequisites

1. Contract deployed on Cosmos Hub with contract address
2. Your wallet address with funds for gas fees
3. `gaiad` CLI configured and connected to Cosmos Hub

### Step 1: Check if Asset Already Exists

First, verify the denom and ticker aren't already registered:

```bash
# Check by denom
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_asset_by_denom":{"denom":"ibc/E3D12B649F5612345678901234567890123456789"}}'

# Check by ticker
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_asset_by_ticker":{"ticker":"ATOM"}}'
```

If you get "not found" responses, you can proceed.

### Step 2: Prepare Asset Information

Gather the following information for your asset:

- **denom**: Full denomination (e.g., `uatom`, `ibc/...`)
- **name**: Human-readable name (e.g., "Cosmos Hub")
- **ticker**: Short ticker (e.g., "ATOM") - will be stored uppercase
- **image_url**: HTTPS URL to asset logo (optional)
- **description**: Brief description of the asset (optional)
- **website**: Official website URL (optional)
- **x**: Twitter/X profile URL (optional)
- **discord**: Discord invite URL (optional)
- **telegram**: Telegram group URL (optional)
- **decimals**: Number of decimal places (optional, defaults to 6)

### Step 3: Add the Asset

Execute the `AddAsset` message:

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_asset": {
    "denom": "uatom",
    "name": "Cosmos Hub",
    "ticker": "ATOM",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/cosmoshub/images/atom.png",
    "description": "The native staking token of Cosmos Hub",
    "website": "https://cosmos.network",
    "x": "https://x.com/cosmos",
    "discord": "https://discord.gg/cosmosnetwork",
    "telegram": "https://t.me/cosmosnetwork",
    "decimals": 6
  }
}' --from <YOUR_KEY> --gas auto --gas-adjustment 1.3 -y
```

### Step 4: Verify the Addition

Query the asset to confirm it was added:

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"get_asset_by_denom":{"denom":"uatom"}}'
```

You should see the complete asset entry with your information.

### Step 5: List Assets to See It in the Registry

```bash
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{"list_assets":{"limit":20}}'
```

Your new asset should appear in the list.

## Examples: Different Asset Types

### Native Token Example

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_asset": {
    "denom": "uatom",
    "name": "Cosmos Hub",
    "ticker": "ATOM",
    "decimals": 6
  }
}' --from mykey --gas auto --gas-adjustment 1.3 -y
```

### IBC Token Example

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_asset": {
    "denom": "ibc/E3D12B649F5612345678901234567890123456789",
    "name": "Wrapped Ethereum",
    "ticker": "WETH",
    "image_url": "https://example.com/weth.png",
    "decimals": 18
  }
}' --from mykey --gas auto --gas-adjustment 1.3 -y
```

### Minimal Entry Example

Only required fields (denom, name, ticker):

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_asset": {
    "denom": "token123",
    "name": "Test Token",
    "ticker": "TEST"
  }
}' --from mykey --gas auto --gas-adjustment 1.3 -y
```

## Admin Operations

Only the contract admin can perform these operations:

### Verify an Asset

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "set_verified": {
    "denom": "uatom",
    "verified": true
  }
}' --from admin_key --gas auto --gas-adjustment 1.3 -y
```

### Block an Asset

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "set_blocked": {
    "denom": "token123",
    "blocked": true
  }
}' --from admin_key --gas auto --gas-adjustment 1.3 -y
```

### Remove an Asset

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "remove_asset": {
    "denom": "token123"
  }
}' --from admin_key --gas auto --gas-adjustment 1.3 -y
```

Note: This permanently removes the asset from both the denom and ticker indexes.

## Structured Descriptions

The contract supports extensible structured descriptions that allow adding new fields over time while maintaining backward compatibility.

### Adding Structured Description Fields

```bash
# Add a technical details field
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_structured_description": {
    "denom": "uatom",
    "key": "technical_details",
    "value": "Native staking token with delegations and rewards"
  }
}' --from creator_key --gas auto --gas-adjustment 1.3 -y

# Add a roadmap field
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "add_structured_description": {
    "denom": "uatom",
    "key": "roadmap",
    "value": "Q4 2024: IBC v3, Q1 2025: Liquid staking"
  }
}' --from creator_key --gas auto --gas-adjustment 1.3 -y
```

### Updating Structured Description Fields

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "update_structured_description": {
    "denom": "uatom",
    "key": "technical_details",
    "value": "Updated: Native staking token with advanced delegation features"
  }
}' --from creator_key --gas auto --gas-adjustment 1.3 -y
```

### Querying Structured Descriptions

```bash
# Get specific structured description
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "get_structured_description": {
    "denom": "uatom",
    "key": "technical_details"
  }
}'

# Get all structured descriptions
gaiad query wasm contract-state smart cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "get_all_structured_descriptions": {
    "denom": "uatom"
  }
}'
```

### Common Structured Description Keys

Suggested key names for structured descriptions:
- `technical_details` - Technical specifications
- `roadmap` - Development roadmap
- `use_case` - Primary use cases
- `governance` - Governance information
- `tokenomics` - Token economics details
- `security_audit` - Security audit information
- `partnerships` - Partnership details
- `team` - Team information

### Backward Compatibility

- The original `description` field is maintained for backward compatibility
- Existing clients continue to work without modification
- New clients can use structured descriptions for richer metadata
- Both systems can coexist within the same asset

### Update Your Own Asset

If you created an asset, you can update it:

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "update_asset": {
    "denom": "uatom",
    "name": "Cosmos Hub Updated",
    "image_url": "https://example.com/new-atom.png",
    "description": "Updated description"
  }
}' --from creator_key --gas auto --gas-adjustment 1.3 -y
```

### Admin Update Ticker

Only the contract admin can update the ticker field:

```bash
gaiad tx wasm execute cosmos1rpp8x460lyl4r53afve7z67jddq3uv780f3q0cm7vakc57j43t7qqf0rv5 '{
  "update_asset": {
    "denom": "uatom",
    "ticker": "ATOM2"
  }
}' --from admin_key --gas auto --gas-adjustment 1.3 -y
```

## Common Errors and Solutions

### "Denom already exists"
- The denom is already registered. Use `get_asset_by_denom` to find the existing entry.

### "Ticker already exists"
- The ticker is already registered. Use `get_asset_by_ticker` to find which denom uses it.

### "Structured description key already exists"
- The structured description key is already registered. Use `update_structured_description` to modify it.

### "Structured description key not found"
- The structured description key doesn't exist. Use `add_structured_description` to create it first.

### Gas too low
- Increase gas adjustment: `--gas-adjustment 1.5` or use `--gas 200000`

## Build and Deployment

See previous section for build and deployment instructions using CosmWasm optimizer 0.17.
