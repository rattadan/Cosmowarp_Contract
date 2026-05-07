# User Registry Contract

A CosmWasm smart contract for managing on-chain user profiles with personal and business information.

## Overview

The User Registry contract allows users to create and manage their on-chain profiles, storing personal information, business details, and contact information. This enables identity management and profile discovery across the ecosystem.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   User Registry Contract                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Profile    │  │   Profile    │  │   Profile    │      │
│  │   Creation   │  │   Update     │  │   Deletion   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                  │               │
│         ▼                 ▼                  ▼               │
│  ┌──────────────────────────────────────────────────┐      │
│  │              Storage Layer                        │      │
│  ├──────────────────────────────────────────────────┤      │
│  │ • PROFILES (address → UserProfile)               │      │
│  │   - Personal information                         │      │
│  │   - Business details                             │      │
│  │   - Contact information                          │      │
│  │   - Profile picture (IPFS)                       │      │
│  └──────────────────────────────────────────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Profile Management**: Create and update user profiles
- ✅ **Personal Information**: Name, profile picture (IPFS)
- ✅ **Business Details**: Company name, tax ID
- ✅ **Contact Information**: Email, phone, website
- ✅ **Location Data**: Address and city
- ✅ **Self-Sovereign**: Users control their own data
- ✅ **Privacy**: Optional fields for selective disclosure
- ✅ **Profile Deletion**: Users can delete their profiles

## State Structure

### UserProfile

```rust
pub struct UserProfile {
    pub address: String,                    // User's wallet address
    pub profile_picture_ipfs: Option<String>, // IPFS hash for profile picture
    pub name: Option<String>,               // Display name
    pub company_name: Option<String>,       // Business/company name
    pub address: Option<String>,            // Physical address
    pub city: Option<String>,               // City
    pub phone: Option<String>,              // Phone number
    pub email: Option<String>,              // Email address
    pub website: Option<String>,            // Website URL
    pub tax_id: Option<String>,             // Tax ID / VAT number
}
```

All fields except `address` (wallet address) are optional, allowing users to share only what they're comfortable with.

## Execute Messages

### UpdateProfile

Create or update a user profile. Only updates fields that are provided.

```json
{
  "update_profile": {
    "profile_picture_ipfs": "QmXxxx...",
    "name": "John Doe",
    "company_name": "ACME Corporation",
    "address": "123 Main Street",
    "city": "New York",
    "phone": "+1-555-0123",
    "email": "john@example.com",
    "website": "https://johndoe.com",
    "tax_id": "US123456789"
  }
}
```

**Note:** All fields are optional. Only include fields you want to update.

### DeleteProfile

Delete your profile from the registry.

```json
{
  "delete_profile": {}
}
```

## Query Messages

### GetProfile

Retrieve a user's profile by address.

```json
{
  "get_profile": {
    "address": "cosmos1..."
  }
}
```

**Response:**
```json
{
  "profile": {
    "address": "cosmos1...",
    "profile_picture_ipfs": "QmXxxx...",
    "name": "John Doe",
    "company_name": "ACME Corporation",
    "address": "123 Main Street",
    "city": "New York",
    "phone": "+1-555-0123",
    "email": "john@example.com",
    "website": "https://johndoe.com",
    "tax_id": "US123456789"
  }
}
```

### ProfileExists

Check if a profile exists for an address.

```json
{
  "profile_exists": {
    "address": "cosmos1..."
  }
}
```

**Response:**
```json
true
```

## Deployment Guide

### Prerequisites

1. **Docker** - For running the rust-optimizer
2. **Cosmos CLI** (`gaiad` or `wasmd`) - For deployment
3. **Wallet with funds** - For gas fees

### Step 1: Build Optimized Contract

```bash
cd user_registry

# Build with optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0
```

This creates `artifacts/user_registry.wasm`.

### Step 2: Store Contract Code

```bash
# Set variables
WALLET="my-wallet"
CHAIN_ID="cosmoshub-4"
NODE="https://rpc.cosmos.network:443"
GAS_PRICES="0.025uatom"

# Upload wasm
TX_HASH=$(gaiad tx wasm store artifacts/user_registry.wasm \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES \
  --broadcast-mode sync \
  --output json -y | jq -r '.txhash')

echo "Transaction Hash: $TX_HASH"

# Wait and get Code ID
sleep 6
CODE_ID=$(gaiad query tx $TX_HASH \
  --node $NODE \
  --output json | \
  jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Code ID: $CODE_ID"
```

### Step 3: Instantiate Contract

```bash
# Instantiate (no parameters needed)
INIT_MSG='{}'

gaiad tx wasm instantiate $CODE_ID "$INIT_MSG" \
  --from $WALLET \
  --label "User Registry v1" \
  --admin $(gaiad keys show $WALLET -a) \
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
# Query contract info
gaiad query wasm contract $CONTRACT_ADDRESS --node $NODE
```

## Usage Examples

### Create/Update Profile

#### Basic Profile

```bash
CONTRACT="<your-contract-address>"
WALLET="my-wallet"

UPDATE_MSG='{
  "update_profile": {
    "name": "Alice Smith",
    "email": "alice@example.com"
  }
}'

gaiad tx wasm execute $CONTRACT "$UPDATE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

#### Business Profile

```bash
UPDATE_MSG='{
  "update_profile": {
    "profile_picture_ipfs": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
    "name": "Bob Johnson",
    "company_name": "Johnson Consulting LLC",
    "address": "456 Business Ave, Suite 100",
    "city": "San Francisco",
    "phone": "+1-415-555-0199",
    "email": "bob@johnsonconsulting.com",
    "website": "https://johnsonconsulting.com",
    "tax_id": "US987654321"
  }
}'

gaiad tx wasm execute $CONTRACT "$UPDATE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

#### Partial Update

Only update specific fields:

```bash
UPDATE_MSG='{
  "update_profile": {
    "phone": "+1-555-NEW-PHONE",
    "website": "https://new-website.com"
  }
}'

gaiad tx wasm execute $CONTRACT "$UPDATE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

### Query Profile

```bash
USER_ADDRESS=$(gaiad keys show $WALLET -a)

gaiad query wasm contract-state smart $CONTRACT \
  '{"get_profile":{"address":"'$USER_ADDRESS'"}}' \
  --node $NODE
```

### Check Profile Existence

```bash
gaiad query wasm contract-state smart $CONTRACT \
  '{"profile_exists":{"address":"'$USER_ADDRESS'"}}' \
  --node $NODE
```

### Delete Profile

```bash
DELETE_MSG='{"delete_profile":{}}'

gaiad tx wasm execute $CONTRACT "$DELETE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

## IPFS Profile Picture Integration

### Upload Image to IPFS

```bash
# Using IPFS CLI
ipfs add profile-picture.jpg
# Returns: QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG

# Or use a service like:
# - Pinata (https://pinata.cloud)
# - NFT.Storage (https://nft.storage)
# - Web3.Storage (https://web3.storage)
```

### Update Profile with IPFS Hash

```bash
UPDATE_MSG='{
  "update_profile": {
    "profile_picture_ipfs": "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
  }
}'

gaiad tx wasm execute $CONTRACT "$UPDATE_MSG" \
  --from $WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto -y
```

### Display Profile Picture in Frontend

```javascript
// Construct IPFS gateway URL
const ipfsHash = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";
const imageUrl = `https://ipfs.io/ipfs/${ipfsHash}`;

// Or use other gateways:
// https://cloudflare-ipfs.com/ipfs/${ipfsHash}
// https://gateway.pinata.cloud/ipfs/${ipfsHash}
```

## Use Cases

### 1. Social Identity
Users create profiles to establish their on-chain identity for social dApps.

### 2. Business Verification
Companies register business information for B2B transactions and invoicing.

### 3. Contact Discovery
Enable users to find and connect with each other based on public profiles.

### 4. KYC/KYB Alternative
Self-sovereign identity without centralized KYC, users control their data.

### 5. Invoice Generation
Payment systems can pull user data for automatic invoice generation.

## Privacy Considerations

### What's Public
- All profile information stored on-chain is **publicly readable**
- Anyone can query any address's profile

### Best Practices
1. **Only share what's necessary**: All fields are optional
2. **Use business email**: Consider using a business email instead of personal
3. **IPFS considerations**: Profile pictures on IPFS are public
4. **Tax ID caution**: Only include if required for business purposes
5. **Update regularly**: Keep information current or remove outdated data

### Privacy-Preserving Options
```bash
# Minimal profile (just name)
{
  "update_profile": {
    "name": "Alice"
  }
}

# Business-only profile (no personal info)
{
  "update_profile": {
    "company_name": "ACME Corp",
    "website": "https://acme.com"
  }
}
```

## Integration Example

### TypeScript/JavaScript

```typescript
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

// Update profile
const updateProfileMsg = {
  update_profile: {
    name: "Alice Smith",
    email: "alice@example.com",
    website: "https://alice.com",
    profile_picture_ipfs: "QmXxxx..."
  }
};

await client.execute(
  userAddress,
  contractAddress,
  updateProfileMsg,
  "auto"
);

// Query profile
const profile = await client.queryContractSmart(
  contractAddress,
  { get_profile: { address: userAddress } }
);

if (profile.profile) {
  console.log("Name:", profile.profile.name);
  console.log("Email:", profile.profile.email);
  
  // Display profile picture
  if (profile.profile.profile_picture_ipfs) {
    const imageUrl = `https://ipfs.io/ipfs/${profile.profile.profile_picture_ipfs}`;
    console.log("Profile Picture:", imageUrl);
  }
}

// Check if profile exists
const exists = await client.queryContractSmart(
  contractAddress,
  { profile_exists: { address: userAddress } }
);
```

### React Component Example

```tsx
import { useQuery } from '@tanstack/react-query';

function UserProfile({ address }: { address: string }) {
  const { data: profile } = useQuery({
    queryKey: ['profile', address],
    queryFn: async () => {
      const result = await client.queryContractSmart(
        contractAddress,
        { get_profile: { address } }
      );
      return result.profile;
    }
  });

  if (!profile) return <div>No profile found</div>;

  return (
    <div className="profile-card">
      {profile.profile_picture_ipfs && (
        <img 
          src={`https://ipfs.io/ipfs/${profile.profile_picture_ipfs}`}
          alt={profile.name || 'Profile'}
        />
      )}
      <h2>{profile.name}</h2>
      {profile.company_name && <p>{profile.company_name}</p>}
      {profile.email && <a href={`mailto:${profile.email}`}>{profile.email}</a>}
      {profile.website && <a href={profile.website}>Website</a>}
    </div>
  );
}
```

## Error Handling

### Common Errors

**"Profile not found"**
- The address doesn't have a profile. Use `update_profile` to create one.

**Gas estimation failed**
- Increase gas limit or gas adjustment.

## Field Validation

The contract does **not** validate field formats. Frontend applications should validate:

- **Email**: Valid email format
- **Phone**: Valid phone number format
- **Website**: Valid URL format
- **IPFS Hash**: Valid IPFS CID format
- **Tax ID**: Appropriate format for jurisdiction

## Best Practices

1. **Gradual Updates**: Start with minimal info, add more as needed
2. **IPFS Pinning**: Ensure profile pictures are pinned to prevent loss
3. **Data Accuracy**: Keep information up-to-date
4. **Privacy First**: Only share what's necessary for your use case
5. **Backup Data**: Keep a local copy of your profile information
6. **Test First**: Test profile updates on testnet before mainnet

## Troubleshooting

### Profile Not Updating
```bash
# Check if transaction succeeded
gaiad query tx <TX_HASH> --node $NODE

# Verify profile was updated
gaiad query wasm contract-state smart $CONTRACT \
  '{"get_profile":{"address":"'$USER_ADDRESS'"}}' \
  --node $NODE
```

### IPFS Image Not Loading
```bash
# Try different IPFS gateways
https://ipfs.io/ipfs/<hash>
https://cloudflare-ipfs.com/ipfs/<hash>
https://gateway.pinata.cloud/ipfs/<hash>

# Verify hash is correct
ipfs cat <hash>
```

### Gas Issues
```bash
# Increase gas limit
--gas 200000

# Or increase gas adjustment
--gas-adjustment 1.5
```

## Future Enhancements

- [ ] Profile verification badges
- [ ] Social links (GitHub, LinkedIn, etc.)
- [ ] Reputation scores
- [ ] Profile visibility controls
- [ ] Multi-signature profile updates
- [ ] Profile history/versioning
- [ ] ENS/domain name integration
- [ ] Profile NFT representation

## Security Considerations

1. **No Admin Control**: No admin can modify or delete user profiles
2. **Self-Sovereign**: Only the user can update their own profile
3. **Public Data**: All data is publicly readable on-chain
4. **No Encryption**: Data is stored in plain text
5. **Immutable History**: Blockchain history preserves old profile data

## License

MIT
