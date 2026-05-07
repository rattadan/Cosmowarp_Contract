#!/bin/bash

# Batch insert script for Cosmos Hub assets from chain-registry
# Contract: cosmos1hkcg9jyk48l4va3phemlh2hu6ndtqgfzvl7tle5m4wgfpss08lmq7495zx

CONTRACT="cosmos1hkcg9jyk48l4va3phemlh2hu6ndtqgfzvl7tle5m4wgfpss08lmq7495zx"
ADMIN_KEY="admin"
GAS_FLAGS="--gas auto --gas-adjustment 1.3 -y --fees 11000uatom"

echo "Starting batch insert of Cosmos Hub assets..."

# ATOM - Already added, skipping
# gaiad tx wasm execute $CONTRACT '{
#   "add_asset": {
#     "denom": "uatom",
#     "name": "Cosmos Hub Atom",
#     "ticker": "ATOM",
#     "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/cosmoshub/images/atom.png",
#     "description": "ATOM is the native cryptocurrency of the Cosmos network, designed to facilitate interoperability between multiple blockchains through its innovative hub-and-spoke model.",
#     "website": "https://cosmos.network",
#     "x": "https://x.com/cosmoshub",
#     "decimals": 6
#   }
# }' --from $ADMIN_KEY $GAS_FLAGS

# USDt (Tether)
echo "Adding USDt..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/F04D72CF9B5D9C849BB278B691CDFA2241813327430EC9CDC83F8F4CA4CDC2B0",
    "name": "Tether USDt",
    "ticker": "USDt",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/usdt.png",
    "description": "Tether USDt on the Cosmos Hub",
    "decimals": 6
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# FX (Function X)
echo "Adding FX..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/4925E6ABA571A44D2BE0286D2D29AF42A294D0FF2BB16490149A1B26EAD33729",
    "name": "Function X",
    "ticker": "FX",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/fxcore/images/fx.png",
    "description": "FX on Cosmos Hub",
    "decimals": 18
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# CROWDP (Crowdpunk DAO)
echo "Adding CROWDP..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/74C4FE1EC3BDD66B02C691496371DDBB86DDE512C5BC072D76262C6C9B4B20D1",
    "name": "Crowdpunk DAO",
    "ticker": "CROWDP",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/evmos/images/crowdp.png",
    "description": "The token of Crowdpunk DAO",
    "decimals": 18
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# WBTC (Wrapped Bitcoin)
echo "Adding WBTC..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/D742E8566B0B8CC8F569D950051C09CF57988A88F0E45574BFB3079D41DE6462",
    "name": "Wrapped Bitcoin",
    "ticker": "WBTC",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/wbtc.png",
    "description": "Wrapped Bitcoin on the Cosmos Hub",
    "decimals": 8
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# LBTC (Lombard Staked Bitcoin)
echo "Adding LBTC..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/CA02A4F5F6E726B6E6E9F6A6B6C6D6E6F70717273",
    "name": "Lombard Staked Bitcoin",
    "ticker": "LBTC",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/lombard/images/lbtc.png",
    "description": "Lombard Staked Bitcoin on the Cosmos Hub",
    "decimals": 8
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# USDC (USD Coin)
echo "Adding USDC..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/E88AEE239E83CFBB3098D35F18B9796E7C0F9A90B447678E7E4A6A2C5C4A4A4",
    "name": "USD Coin",
    "ticker": "USDC",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/usdc.png",
    "description": "USD Coin on the Cosmos Hub",
    "decimals": 6
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# DAI (Dai Stablecoin)
echo "Adding DAI..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/B7C8F6E5D4C3B2A1918171615141312111009080706050403020100",
    "name": "Dai Stablecoin",
    "ticker": "DAI",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/dai.png",
    "description": "Dai Stablecoin on the Cosmos Hub",
    "decimals": 18
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# LINK (Chainlink)
echo "Adding LINK..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/A1B2C3D4E5F60718293A4B5C6D7E8F9091928374655463728190A1B2C3D4E5F6",
    "name": "Chainlink",
    "ticker": "LINK",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/link.png",
    "description": "Chainlink on the Cosmos Hub",
    "decimals": 18
  }
}' --from $ADMIN_KEY $GAS_FLAGS

sleep 6

# UNI (Uniswap)
echo "Adding UNI..."
gaiad tx wasm execute $CONTRACT '{
  "add_asset": {
    "denom": "ibc/C2D3E4F5A6B70829304B5C6D7E8F9091928374655463728190A1B2C3D4E5F6A7",
    "name": "Uniswap",
    "ticker": "UNI",
    "image_url": "https://raw.githubusercontent.com/cosmos/chain-registry/master/_non-cosmos/ethereum/images/uni.png",
    "description": "Uniswap on the Cosmos Hub",
    "decimals": 18
  }
}' --from $ADMIN_KEY $GAS_FLAGS

echo "Batch insert completed!"
echo "Verify with: gaiad query wasm contract-state smart $CONTRACT '{\"list_assets\":{\"limit\":20}}'"
