# Chat Module Deployment Guide

## 🚀 Deploy to Cosmos Hub (cosmoshub-4)

### Prerequisites

1. **Install gaiad** (Cosmos Hub CLI)
```bash
# Download from https://github.com/cosmos/gaia/releases
# Or install via package manager
```

2. **Create/Import Wallet**
```bash
# Create new wallet
gaiad keys add my-wallet

# Or import existing wallet
gaiad keys add my-wallet --recover
```

3. **Get ATOM tokens** for gas fees
- Minimum ~0.5 ATOM recommended for deployment

---

## 📦 Step 1: Build Optimized Contract

```bash
cd chat_module

# Run optimizer (creates optimized wasm in artifacts/)
./optimize.sh

# Or run docker command directly:
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0
```

**Expected output:**
```
artifacts/chat_module.wasm  (~150-200 KB optimized)
```

---

## 📤 Step 2: Upload Contract to Chain

```bash
# Set variables
WALLET="my-wallet"
CHAIN_ID="cosmoshub-4"
NODE="https://rpc.cosmos.network:443"
GAS_PRICES="0.025uatom"

# Upload wasm
gaiad tx wasm store artifacts/chat_module.wasm \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode sync \
  -y

# Wait for transaction to be included in a block
# Then query the code ID from the transaction hash
TX_HASH="<your-tx-hash>"
gaiad query tx $TX_HASH --node $NODE
```

**Save the Code ID** from the transaction output!

---

## 🎯 Step 3: Instantiate Contract

```bash
# Set your code ID
CODE_ID=<your-code-id>

# Set admin address (your wallet address)
ADMIN=$(gaiad keys show $WALLET -a)

# Create instantiate message
INIT_MSG='{
  "admin": "'$ADMIN'"
}'

# Instantiate contract
gaiad tx wasm instantiate $CODE_ID "$INIT_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --label "chat-module-v1" \
  --admin $ADMIN \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode sync \
  -y

# Query contract address
gaiad query wasm list-contract-by-code $CODE_ID --node $NODE --output json
```

**Save the Contract Address!**

---

## 🔧 Step 4: Configure Frontend

Add the contract address to your `.env.local`:

```bash
# In multichain-pay root directory
echo "NEXT_PUBLIC_CHAT_CONTRACT_ADDRESS=cosmos1..." >> .env.local
echo "NEXT_PUBLIC_REST_ENDPOINT=https://rest.cosmos.network" >> .env.local
echo "NEXT_PUBLIC_RPC_ENDPOINT=https://rpc.cosmos.network" >> .env.local
```

---

## ✅ Step 5: Test the Contract

### Create a Test Group
```bash
CONTRACT="<your-contract-address>"

# Create public group
CREATE_GROUP_MSG='{
  "create_group": {
    "group_id": "general",
    "name": "General Chat",
    "description": "Public discussion",
    "logo_url": null,
    "is_public": true
  }
}'

gaiad tx wasm execute $CONTRACT "$CREATE_GROUP_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  -y
```

### Send a Test Message
```bash
SEND_MSG='{
  "send_message": {
    "group_id": "general",
    "content": "Hello from Cosmos Hub!",
    "reply_to": null
  }
}'

gaiad tx wasm execute $CONTRACT "$SEND_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  -y
```

### Query Messages
```bash
QUERY_MSG='{"list_messages":{"group_id":"general","limit":10}}'
gaiad query wasm contract-state smart $CONTRACT "$QUERY_MSG" --node $NODE
```

---

## 📊 Useful Queries

### Get Contract Info
```bash
gaiad query wasm contract $CONTRACT --node $NODE
```

### Get Contract State
```bash
gaiad query wasm contract-state all $CONTRACT --node $NODE
```

### List Public Groups
```bash
QUERY='{"list_public_groups":{"limit":100}}'
gaiad query wasm contract-state smart $CONTRACT "$QUERY" --node $NODE
```

### Get User's Groups
```bash
USER_ADDR="cosmos1..."
QUERY='{"get_user_groups":{"user":"'$USER_ADDR'","limit":100}}'
gaiad query wasm contract-state smart $CONTRACT "$QUERY" --node $NODE
```

---

## 🔐 Security Considerations

1. **Admin Key Security**
   - Store admin private key securely
   - Consider using a hardware wallet for mainnet
   - Admin can delete any group and message

2. **Contract Upgrades**
   - Contract is instantiated with admin
   - Can migrate to new code if needed
   - Keep admin address secure

3. **Gas Costs**
   - Store operations cost gas
   - Large messages cost more
   - Consider message size limits

---

## 🐛 Troubleshooting

### "Out of gas" Error
```bash
# Increase gas limit
--gas 2000000
# Or increase gas adjustment
--gas-adjustment 1.5
```

### "Insufficient funds" Error
```bash
# Check balance
gaiad query bank balances $(gaiad keys show $WALLET -a) --node $NODE

# Get testnet tokens from faucet (for testnet)
```

### Contract Query Fails
```bash
# Verify contract address
gaiad query wasm contract $CONTRACT --node $NODE

# Check if contract is instantiated
gaiad query wasm list-contract-by-code $CODE_ID --node $NODE
```

---

## 📝 Contract Features Summary

- ✅ Public & Private Groups
- ✅ Direct Messages (DM) with dual encryption
- ✅ Message voting (thumbs up/down)
- ✅ User preferences (color, bio)
- ✅ Group logos
- ✅ Message editing & deletion
- ✅ Admin moderation
- ✅ Reply threading

---

## 🌐 Mainnet vs Testnet

### Testnet (theta-testnet-001)
```bash
CHAIN_ID="theta-testnet-001"
NODE="https://rpc.sentry-01.theta-testnet.polypore.xyz:443"
```

### Mainnet (cosmoshub-4)
```bash
CHAIN_ID="cosmoshub-4"
NODE="https://rpc.cosmos.network:443"
```

---

## 📚 Additional Resources

- [CosmWasm Docs](https://docs.cosmwasm.com/)
- [Cosmos Hub](https://hub.cosmos.network/)
- [Contract Schema](./schema/chat_module.json)
- [Frontend Guide](../CHAT_FRONTEND_README.md)

---

**Deployment Complete!** 🎉

Your chat contract is now live on Cosmos Hub. Update your frontend `.env.local` with the contract address and start chatting!
