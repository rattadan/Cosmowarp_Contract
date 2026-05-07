# OTC Escrow Contract

A CosmWasm smart contract for peer-to-peer atomic swaps (OTC trades) between two parties.
cosmos1eck8cffl4llgp7a0krz6c28egmzgy0rpny2se5usfaw64vchfzmsnzx6pl
code id 366

docker optimizer 0.17 
rustc 1.91.1 (ed61e7d7e 2025-11-07)
cargo 1.91.1 (ea2d97820 2025-10-10)


## Overview

This contract allows users to create "vaults" where they offer one token in exchange for another. For example, a user can offer 2 ATOM in exchange for 4 USDC. Any counterparty can then fill the other side of the trade, and the swap executes atomically.

## Features

- **Create OTC Vault**: Creator defines what they offer and what they want, funding their side immediately
- **Atomic Swap**: When counterparty funds their side, both parties receive their tokens in a single transaction
- **Cancel Vault**: Creator can cancel and get their funds back (only before counterparty funds)
- **Public Listing**: Open vaults are publicly queryable so anyone can browse available trades
- **Completed History**: Completed trades are stored separately for historical reference
- **Same-Denom Prevention**: Cannot create a vault to swap the same token (e.g., ATOM for ATOM)
- **Expiry Support**: Optional expiration time for vaults

## Vault Lifecycle

```
┌─────────────┐
│   Created   │  (Creator sends CreateVault with funds)
└──────┬──────┘
       │ (funds attached)
       ▼
┌─────────────┐
│    Open     │  (Listed publicly, waiting for counterparty)
└──────┬──────┘
       │
       ├────────────────────────────┐
       │ (Counterparty funds)       │ (Creator cancels)
       ▼                            ▼
┌─────────────┐              ┌─────────────┐
│  Completed  │              │  Cancelled  │
│ (Swap done) │              │  (Refunded) │
└─────────────┘              └─────────────┘
```

## Messages

### Execute Messages

#### CreateVault
Create a new OTC vault. Must send the offer amount with this message.

```json
{
  "create_vault": {
    "vault_id": "vault_001",
    "ask_amount": "4000000",
    "ask_denom": "uusdc",
    "description": "2 ATOM for 4 USDC",
    "expires_in": 86400
  }
}
```
Send with: `2000000uatom`

#### FundVault
Fund the counterparty side of a vault. Must send exact ask_amount in ask_denom.

```json
{
  "fund_vault": {
    "vault_id": "vault_001"
  }
}
```
Send with: `4000000uusdc`

Upon success:
- Creator receives 4 USDC
- Counterparty receives 2 ATOM

#### CancelVault
Cancel a vault (creator only, before counterparty funds).

```json
{
  "cancel_vault": {
    "vault_id": "vault_001"
  }
}
```

### Query Messages

#### GetVault
Get a specific vault by ID.

```json
{
  "get_vault": {
    "vault_id": "vault_001"
  }
}
```

#### GetOpenVaults
Get all open vaults (available for funding).

```json
{
  "get_open_vaults": {
    "offer_denom": "uatom",
    "ask_denom": "uusdc",
    "limit": 50
  }
}
```

#### GetCompletedVaults
Get completed vaults (historical).

```json
{
  "get_completed_vaults": {
    "limit": 50
  }
}
```

#### GetCreatorVaults
Get all vaults created by an address.

```json
{
  "get_creator_vaults": {
    "creator": "cosmos1...",
    "status": "open"
  }
}
```

#### GetCounterpartyVaults
Get all vaults where address was counterparty.

```json
{
  "get_counterparty_vaults": {
    "counterparty": "cosmos1..."
  }
}
```

## State

### Vault Structure

```rust
pub struct Vault {
    pub vault_id: String,
    pub creator: String,
    pub counterparty: Option<String>,
    pub offer_amount: u128,
    pub offer_denom: String,
    pub ask_amount: u128,
    pub ask_denom: String,
    pub description: String,
    pub status: VaultStatus,
    pub created_at: u64,
    pub creator_funded_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub expires_at: u64,
}
```

### Vault Status

- `Created` - Initial state (not used in current flow)
- `Open` - Creator has funded, waiting for counterparty
- `Completed` - Both sides funded, swap executed
- `Cancelled` - Creator cancelled, funds returned

## Storage

- `VAULTS` - Map of vault_id → Vault
- `OPEN_VAULTS` - List of open vault IDs (for browsing)
- `COMPLETED_VAULTS` - List of completed vault IDs (historical)
- `CREATOR_VAULTS` - Map of creator address → vault IDs
- `COUNTERPARTY_VAULTS` - Map of counterparty address → vault IDs

## Build

```bash
# Check
cargo check

# Test
cargo test

# Build optimized wasm
cargo build --release --target wasm32-unknown-unknown

# Or use optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.15.0
```

## Deploy

```bash
# Store code
wasmd tx wasm store artifacts/otc_escrow.wasm \
  --from wallet --gas auto --gas-adjustment 1.3 -y

# Instantiate
wasmd tx wasm instantiate $CODE_ID \
  '{"admin": null}' \
  --label "OTC Escrow" \
  --from wallet --admin wallet --gas auto -y
```

## Example Flow

1. **Alice wants to trade 2 ATOM for 4 USDC**
   ```bash
   wasmd tx wasm execute $CONTRACT \
     '{"create_vault":{"vault_id":"alice_001","ask_amount":"4000000","ask_denom":"uusdc","description":"2 ATOM for 4 USDC","expires_in":86400}}' \
     --amount 2000000uatom --from alice -y
   ```

2. **Bob sees the vault and wants to take the trade**
   ```bash
   wasmd tx wasm execute $CONTRACT \
     '{"fund_vault":{"vault_id":"alice_001"}}' \
     --amount 4000000uusdc --from bob -y
   ```

3. **Result**: Alice receives 4 USDC, Bob receives 2 ATOM

4. **Or Alice changes her mind before Bob funds**
   ```bash
   wasmd tx wasm execute $CONTRACT \
     '{"cancel_vault":{"vault_id":"alice_001"}}' \
     --from alice -y
   ```
   Alice gets her 2 ATOM back.

## Security Considerations

- Creator cannot be their own counterparty
- Same-denom swaps are prevented
- Only creator can cancel their vault
- Cancellation only allowed before counterparty funds
- Expired vaults cannot be funded
- Exact amounts required (no partial fills)
