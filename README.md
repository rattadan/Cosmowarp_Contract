# Multichain Pay - Smart Contracts

A comprehensive suite of CosmWasm smart contracts for building decentralized payment and registry systems on Cosmos-based blockchains.

## 📋 Overview

This repository contains six production-ready smart contracts that work together to create a complete decentralized payment ecosystem:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Multichain Pay Ecosystem                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Asset     │  │     dApp     │  │     User     │          │
│  │   Registry   │  │   Registry   │  │   Registry   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         │                 │                  │                   │
│         └─────────────────┴──────────────────┘                   │
│                           │                                       │
│                           ▼                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Payment    │  │     OTC      │  │     Chat     │          │
│  │   Escrow     │  │    Escrow    │  │    Module    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 Contracts

### 1. [Asset Registry](./asset_registry/README.md)
**On-chain asset list repository with verification system**

- 📝 Register any token with metadata
- ⭐ Admin verification for trusted assets
- 🔍 Query by denom or ticker
- 🗂️ Extensible structured descriptions
- 🌐 Social links and branding

**Deployed:** `cosmos1hkcg9jyk48l4va3phemlh2hu6ndtqgfzvl7tle5m4wgfpss08lmq7495zx`

[View Documentation →](./asset_registry/README.md)

---

### 2. [dApp Registry](./dApp_registry/README.md)
**Decentralized application registry with star-based ranking**

- 🎯 Register dApps with full metadata
- ⭐ Community star ranking (10 stars per user)
- 🔄 Star redelegation between dApps
- ✅ Verification and moderation system
- 📊 Leaderboard and discovery

[View Documentation →](./dApp_registry/README.md) | [Deployment Guide →](./dApp_registry/DEPLOYMENT.md)

---

### 3. [User Registry](./user_registry/README.md)
**On-chain user profiles and identity management**

- 👤 Personal and business profiles
- 🖼️ IPFS profile pictures
- 📧 Contact information (email, phone, website)
- 🏢 Business details (company, tax ID)
- 🔒 Self-sovereign data control

[View Documentation →](./user_registry/README.md)

---

### 4. [Payment Escrow](./invoiced_payment_escrow/README.md)
**Secure invoice-based payment escrow system**

- 💰 Invoice creation and funding
- 🔐 Escrow protection for transactions
- ✅ Receiver-controlled release/refund
- ⏰ Expiry support
- 📊 Status tracking

[View Documentation →](./invoiced_payment_escrow/README.md)

---

### 5. [OTC Escrow](./otc_escrow/README.md)
**Peer-to-peer atomic swap for OTC trading**

- 🔄 Atomic token swaps
- 💱 Create buy/sell offers
- 🔒 Trustless execution
- ⏰ Expiration support
- 📜 Trade history

**Deployed:** `cosmos1eck8cffl4llgp7a0krz6c28egmzgy0rpny2se5usfaw64vchfzmsnzx6pl` (Code ID: 366)

[View Documentation →](./otc_escrow/README.md)

---

### 6. [Chat Module](./chat_module/README.md)
**On-chain messaging with groups and encrypted DMs**

- 💬 Public and private groups
- 🔐 Encrypted direct messages
- ✏️ Message editing and deletion
- 💬 Reply threading
- 👥 Group management

[View Documentation →](./chat_module/README.md) | [Deployment Guide →](./chat_module/DEPLOYMENT.md)

---

## 🏗️ Architecture

### System Integration

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend dApp                        │
└────────────┬────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│                    CosmWasm Contracts                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Asset Registry ──┐                                          │
│                   ├──► Payment Escrow ──► User Registry     │
│  dApp Registry ───┘                                          │
│                                                               │
│  OTC Escrow ──────────► Asset Registry                      │
│                                                               │
│  Chat Module ─────────► User Registry                       │
│                                                               │
└─────────────────────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────────┐
│                    Cosmos Hub / IBC                          │
└─────────────────────────────────────────────────────────────┘
```

### Contract Interactions

- **Payment Escrow** can reference **User Registry** for profile data
- **OTC Escrow** uses **Asset Registry** for token metadata
- **Chat Module** can integrate with **User Registry** for user info
- **dApp Registry** showcases applications using these contracts

## 🛠️ Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install Docker (for optimizer)
# https://docs.docker.com/get-docker/
```

### Building Contracts

#### Individual Contract

```bash
cd <contract-folder>

# Development build
cargo build

# Run tests
cargo test

# Optimized build
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0
```

#### Build All Contracts

```bash
# From contracts root directory
for dir in asset_registry chat_module dApp_registry otc_escrow payment_escrow user_registry; do
  echo "Building $dir..."
  cd $dir
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/optimizer:0.17.0
  cd ..
done
```

## 📦 Deployment

### Quick Start

1. **Build optimized WASM**
   ```bash
   cd <contract-folder>
   docker run --rm -v "$(pwd)":/code \
     --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
     --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
     cosmwasm/optimizer:0.17.0
   ```

2. **Store code on-chain**
   ```bash
   gaiad tx wasm store artifacts/<contract>.wasm \
     --from <wallet> \
     --gas auto \
     --gas-adjustment 1.3 \
     --gas-prices 0.025uatom \
     --chain-id cosmoshub-4 \
     --node https://rpc.cosmos.network:443 -y
   ```

3. **Instantiate contract**
   ```bash
   gaiad tx wasm instantiate <code-id> '<init-msg>' \
     --from <wallet> \
     --label "<contract-name>" \
     --admin <admin-address> \
     --gas auto \
     --chain-id cosmoshub-4 \
     --node https://rpc.cosmos.network:443 -y
   ```

### Deployment Guides

Each contract has detailed deployment instructions:

- [Asset Registry Deployment](./asset_registry/README.md#build-and-deployment)
- [dApp Registry Deployment](./dApp_registry/DEPLOYMENT.md)
- [User Registry Deployment](./user_registry/README.md#deployment-guide)
- [Payment Escrow Deployment](./invoiced_payment_escrow/README.md#deployment-guide)
- [OTC Escrow Deployment](./otc_escrow/README.md#deploy)
- [Chat Module Deployment](./chat_module/DEPLOYMENT.md)

## 🌐 Supported Networks

### Mainnet
- **Cosmos Hub** (cosmoshub-4)
  - RPC: `https://rpc.cosmos.network:443`
  - REST: `https://rest.cosmos.network`

### Testnet
- **Theta Testnet** (theta-testnet-001)
  - RPC: `https://rpc.sentry-01.theta-testnet.polypore.xyz:443`

### Local Development
- **LocalOsmosis** or **LocalCosmos**
- See [LocalOsmosis docs](https://docs.osmosis.zone/developing/dapps/get_started/localosmosis.html)

## 🧪 Testing

### Unit Tests

```bash
cd <contract-folder>
cargo test
```

### Integration Tests

```bash
# Run all tests with coverage
cargo tarpaulin --out Html
```

### Test on Testnet

1. Deploy to testnet
2. Run test scripts in `scripts/` folders
3. Verify functionality before mainnet deployment

## 📊 Gas Costs (Approximate)

| Operation | Gas | Cost (0.025 uatom/gas) |
|-----------|-----|------------------------|
| Store Code | ~2,000,000 | ~0.05 ATOM |
| Instantiate | ~200,000 | ~0.005 ATOM |
| Add Asset/dApp | ~150,000 | ~0.004 ATOM |
| Create Invoice | ~100,000 | ~0.0025 ATOM |
| Fund Invoice | ~150,000 | ~0.004 ATOM |
| Send Message | ~120,000 | ~0.003 ATOM |
| Update Profile | ~100,000 | ~0.0025 ATOM |

*Note: Actual costs vary based on network congestion and message complexity*

## 🔐 Security

### Audit Status
- ⚠️ **Not audited** - Use at your own risk
- Recommended for testnet and development only
- Consider professional audit before mainnet deployment

### Security Features
- ✅ Access control (admin, creator, user-specific)
- ✅ Input validation
- ✅ State consistency checks
- ✅ Reentrancy protection (via CosmWasm)
- ✅ Comprehensive unit tests

### Best Practices
1. Test thoroughly on testnet
2. Start with small amounts
3. Verify contract addresses
4. Use hardware wallets for admin keys
5. Monitor contract activity

## 🤝 Contributing

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```
3. **Make changes and test**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
4. **Commit and push**
   ```bash
   git commit -m "Add new feature"
   git push origin feature/my-new-feature
   ```
5. **Create Pull Request**

### Code Standards
- Follow Rust best practices
- Add unit tests for new features
- Update documentation
- Run `cargo fmt` and `cargo clippy`
- Keep gas costs reasonable

## 📚 Resources

### CosmWasm
- [CosmWasm Documentation](https://docs.cosmwasm.com/)
- [CosmWasm Book](https://book.cosmwasm.com/)
- [CosmWasm Examples](https://github.com/CosmWasm/cw-examples)

### Cosmos SDK
- [Cosmos SDK Docs](https://docs.cosmos.network/)
- [Cosmos Hub](https://hub.cosmos.network/)
- [IBC Protocol](https://ibcprotocol.org/)

### Tools
- [CosmJS](https://github.com/cosmos/cosmjs) - JavaScript client
- [Telescope](https://github.com/osmosis-labs/telescope) - TypeScript codegen
- [LocalOsmosis](https://docs.osmosis.zone/developing/dapps/get_started/localosmosis.html) - Local testnet

## 🗺️ Roadmap

### Phase 1: Core Contracts ✅
- [x] Asset Registry
- [x] User Registry
- [x] Payment Escrow
- [x] OTC Escrow
- [x] dApp Registry
- [x] Chat Module

### Phase 2: Enhancements 🚧
- [ ] Multi-signature support
- [ ] Governance integration
- [ ] Cross-chain IBC support
- [ ] Advanced querying
- [ ] Event subscriptions

### Phase 3: Ecosystem 📋
- [ ] Frontend dApp
- [ ] Mobile applications
- [ ] Analytics dashboard
- [ ] Developer SDK
- [ ] API documentation

## 📄 License

All contracts are licensed under MIT License.

## 💬 Support

- **Issues**: [GitHub Issues](https://github.com/your-repo/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-repo/discussions)
- **Documentation**: See individual contract READMEs

## 🎯 Quick Links

| Contract | README | Deployment | Tests |
|----------|--------|------------|-------|
| Asset Registry | [📖](./asset_registry/README.md) | [🚀](./asset_registry/README.md#build-and-deployment) | [✅](./asset_registry/src/contract.rs#L587) |
| dApp Registry | [📖](./dApp_registry/README.md) | [🚀](./dApp_registry/DEPLOYMENT.md) | [✅](./dApp_registry/src/contract.rs) |
| User Registry | [📖](./user_registry/README.md) | [🚀](./user_registry/README.md#deployment-guide) | [✅](./user_registry/src/contract.rs#L152) |
| Payment Escrow | [📖](./invoiced_payment_escrow/README.md) | [🚀](./invoiced_payment_escrow/README.md#deployment-guide) | [✅](./invoiced_payment_escrow/src/contract.rs#L384) |
| OTC Escrow | [📖](./otc_escrow/README.md) | [🚀](./otc_escrow/README.md#deploy) | [✅](./otc_escrow/src/contract.rs) |
| Chat Module | [📖](./chat_module/README.md) | [🚀](./chat_module/DEPLOYMENT.md) | [✅](./chat_module/src/contract.rs) |

---

**Built with ❤️ for the Cosmos ecosystem**
