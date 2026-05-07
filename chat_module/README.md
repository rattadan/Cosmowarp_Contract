# Chat Module - CosmWasm Contract

On-chain chat module with public groups, private groups, and encrypted direct messages.

## Features

- ✅ **Public Groups**: Anyone can create and join public chat groups
- ✅ **Private Groups**: Create private groups with admin control
- ✅ **Direct Messages**: Encrypted 1-on-1 conversations (frontend encryption)
- ✅ **Message Editing**: Users can edit their own messages (marked as "modified")
- ✅ **Message Deletion**: Users can delete their own messages, admins can delete any message in their group
- ✅ **Timestamps**: All messages include creation and modification timestamps
- ✅ **Reply Threading**: Messages can reply to other messages
- ✅ **No Limits**: Unlimited messages per group

## Contract Structure

### State

- **ChatGroup**: Group metadata (name, admin, public/private, message count)
- **Message**: Message content, sender, timestamp, modified flag
- **Config**: Contract configuration (admin)

### Execute Messages

```rust
CreateGroup { group_id, name, description, is_public }
DeleteGroup { group_id }
SendMessage { group_id, content, reply_to }
EditMessage { group_id, message_id, new_content }
DeleteMessage { group_id, message_id }
SendDirectMessage { recipient, encrypted_content }
UpdateConfig { admin }
```

### Query Messages

```rust
GetConfig {}
GetGroup { group_id }
ListPublicGroups { start_after, limit }
GetUserGroups { user, start_after, limit }
GetMessage { group_id, message_id }
ListMessages { group_id, start_after, limit }
GetDirectMessages { counterparty, start_after, limit }
```

## Access Control

| Action | Permission |
|--------|------------|
| Create Group | Anyone |
| Delete Group | Group admin only |
| Send Message | Anyone (public groups) |
| Edit Message | Message sender only |
| Delete Message | Message sender OR group admin |

## Direct Messages (DMs)

Direct messages use a special group ID format: `dm:{addr1}:{addr2}` (sorted alphabetically).

**Encryption Flow:**
1. Frontend encrypts message with recipient's public key (derived from wallet address)
2. Contract stores encrypted string
3. Only recipient can decrypt with their private key

**Recommended:** Use `secp256k1` encryption (native to Cosmos wallets)

## Building

```bash
# Install Rust and wasm32 target
rustup target add wasm32-unknown-unknown

# Build optimized wasm
chmod +x scripts/build.sh
./scripts/build.sh
```

## Deployment

```bash
# Store the contract
wasmd tx wasm store artifacts/chat_module.wasm --from <your-key> --gas auto --gas-adjustment 1.3

# Instantiate
wasmd tx wasm instantiate <code-id> '{"admin":"cosmos1..."}' --from <your-key> --label "chat-module" --gas auto
```

## Usage Examples

### Create a Public Group

```bash
wasmd tx wasm execute <contract-addr> '{
  "create_group": {
    "group_id": "general",
    "name": "General Chat",
    "description": "Public discussion",
    "is_public": true
  }
}' --from <your-key>
```

### Send a Message

```bash
wasmd tx wasm execute <contract-addr> '{
  "send_message": {
    "group_id": "general",
    "content": "Hello, world!",
    "reply_to": null
  }
}' --from <your-key>
```

### Query Public Groups

```bash
wasmd query wasm contract-state smart <contract-addr> '{
  "list_public_groups": {
    "start_after": null,
    "limit": 50
  }
}'
```

### Query Messages

```bash
wasmd query wasm contract-state smart <contract-addr> '{
  "list_messages": {
    "group_id": "general",
    "start_after": null,
    "limit": 50
  }
}'
```

### Send Encrypted DM

```javascript
// Frontend example (using tweetnacl or similar)
const recipientPubKey = derivePublicKey(recipientAddress);
const encryptedContent = encrypt(message, recipientPubKey);

await executeContract({
  send_direct_message: {
    recipient: "cosmos1...",
    encrypted_content: encryptedContent
  }
});
```

## Frontend Integration

### Recommended Libraries

- **Encryption**: `tweetnacl`, `libsodium.js`, or `noble-secp256k1`
- **Wallet**: `@cosmos-kit/react` or `@interchain-kit/react`
- **Queries**: `@tanstack/react-query`

### Public Key Derivation

```javascript
import { pubkeyToAddress } from '@cosmjs/amino';

// Get public key from wallet
const accounts = await wallet.getAccounts();
const pubkey = accounts[0].pubkey;

// Use for encryption
const encryptedMsg = encrypt(message, pubkey);
```

## Future Enhancements

- [ ] Message reactions/emojis
- [ ] Pinned messages
- [ ] Group member management (kick/ban)
- [ ] Message fees (spam prevention)
- [ ] File attachments (IPFS hashes)
- [ ] Read receipts
- [ ] Typing indicators (off-chain)

## License

MIT
