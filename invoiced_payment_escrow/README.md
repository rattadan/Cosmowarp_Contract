# Payment Escrow Contract

A CosmWasm smart contract for secure payment escrow with invoice management, enabling trustless transactions between parties.

## Overview

The Payment Escrow contract allows users to create invoices that can be funded by payers. The receiver controls when to release funds (completing the payment) or refund them back to the sender. This creates a secure escrow mechanism for peer-to-peer payments.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Payment Escrow Contract                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Invoice    │  │    Fund      │  │   Release/   │      │
│  │   Creation   │  │   Invoice    │  │   Refund     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                 │                  │               │
│         ▼                 ▼                  ▼               │
│  ┌──────────────────────────────────────────────────┐      │
│  │              Storage Layer                        │      │
│  ├──────────────────────────────────────────────────┤      │
│  │ • INVOICES (invoice_id → Invoice)                │      │
│  │ • RECEIVER_INVOICES (receiver → invoice_ids)     │      │
│  │ • SENDER_INVOICES (sender → invoice_ids)         │      │
│  │ • CONFIG (user registry reference)               │      │
│  └──────────────────────────────────────────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Invoice Creation**: Receivers create invoices with amount and reference
- ✅ **Secure Funding**: Senders fund invoices with exact amounts
- ✅ **Escrow Protection**: Funds held in contract until released or refunded
- ✅ **Release Control**: Only receiver can release funds to themselves
- ✅ **Refund Capability**: Receiver can refund sender if needed
- ✅ **Expiry Support**: Optional expiration time for invoices
- ✅ **Status Tracking**: Track invoice lifecycle (Pending → Funded → Completed/Refunded)
- ✅ **Multi-Denom**: Support any token denomination

The architecture simulates a two-party handshake (A funds, B withdraws) while ensuring the safety of the transaction.

## Invoice Lifecycle

```
┌─────────────┐
│   Pending   │  (Receiver creates invoice)
└──────┬──────┘
       │ (Sender funds with exact amount)
       ▼
┌─────────────┐
│   Funded    │  (Funds held in escrow)
└──────┬──────┘
       │
       ├────────────────────────────┐
       │ (Receiver releases)        │ (Receiver refunds)
       ▼                            ▼
┌─────────────┐              ┌─────────────┐
│  Completed  │              │  Refunded   │
│ (Funds sent │              │ (Funds sent │
│ to receiver)│              │ to sender)  │
└─────────────┘              └─────────────┘
       OR                           OR
┌─────────────┐              ┌─────────────┐
│  Cancelled  │              │   Expired   │
│ (Before     │              │ (Time limit │
│  funding)   │              │  exceeded)  │
└─────────────┘              └─────────────┘
```

## State Structure

### Invoice

```rust
pub struct Invoice {
    pub invoice_id: String,        // Unique identifier
    pub receiver: String,          // Invoice creator (receives payment)
    pub sender: Option<String>,    // Payer (set when funded)
    pub amount: u128,              // Payment amount
    pub denom: String,             // Token denomination (e.g., "uatom")
    pub reference: String,         // Payment reference/memo
    pub status: InvoiceStatus,     // Current status
    pub created_at: u64,           // Creation timestamp
    pub funded_at: Option<u64>,    // Funding timestamp
    pub completed_at: Option<u64>, // Completion timestamp
    pub expires_at: u64,           // Expiration timestamp (0 = no expiry)
}
```

### Invoice Status

```rust
pub enum InvoiceStatus {
    Pending,    // Waiting for funding
    Funded,     // Funded, awaiting release/refund
    Completed,  // Released to receiver
    Refunded,   // Refunded to sender
    Cancelled,  // Cancelled by receiver before funding
}
```

## Execute Messages

### CreateInvoice

Create a new invoice (receiver).

```json
{
  "create_invoice": {
    "invoice_id": "INV-2024-001",
    "amount": "1000000",
    "denom": "uatom",
    "reference": "Payment for services rendered",
    "expires_in": 86400
  }
}
```

**Parameters:**
- `invoice_id`: Unique identifier for the invoice
- `amount`: Payment amount in base units (e.g., 1000000 = 1 ATOM)
- `denom`: Token denomination (e.g., "uatom", "uusdc")
- `reference`: Description or reference for the payment
- `expires_in`: Optional expiration time in seconds (null = no expiry)

### FundInvoice

Fund an existing invoice (sender). Must send exact amount with transaction.

```json
{
  "fund_invoice": {
    "invoice_id": "INV-2024-001"
  }
}
```

**Command:**
```bash
gaiad tx wasm execute $CONTRACT '{"fund_invoice":{"invoice_id":"INV-2024-001"}}' \
  --amount 1000000uatom \
  --from sender_wallet \
  --gas auto -y
```

### ReleaseFunds

Release escrowed funds to receiver (receiver only).

```json
{
  "release_funds": {
    "invoice_id": "INV-2024-001"
  }
}
```

### RefundFunds

Refund escrowed funds to sender (receiver only).

```json
{
  "refund_funds": {
    "invoice_id": "INV-2024-001"
  }
}
```

### CancelInvoice

Cancel an unfunded invoice (receiver only).

```json
{
  "cancel_invoice": {
    "invoice_id": "INV-2024-001"
  }
}
```

## Query Messages

### GetInvoice

Retrieve a specific invoice by ID.

```json
{
  "get_invoice": {
    "invoice_id": "INV-2024-001"
  }
}
```

### GetReceiverInvoices

Get all invoices for a receiver, optionally filtered by status.

```json
{
  "get_receiver_invoices": {
    "receiver": "cosmos1...",
    "status": "Funded"
  }
}
```

**Status options:** `"Pending"`, `"Funded"`, `"Completed"`, `"Refunded"`, `"Cancelled"`, or `null` for all.

### GetSenderInvoices

Get all invoices for a sender (payer).

```json
{
  "get_sender_invoices": {
    "sender": "cosmos1...",
    "status": null
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
2. **Cosmos CLI** (`gaiad` or `wasmd`) - For deployment
3. **Wallet with funds** - For gas fees

### Step 1: Build Optimized Contract

```bash
cd payment_escrow

# Build with optimizer
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0
```

This creates `artifacts/payment_escrow.wasm`.

### Step 2: Store Contract Code

```bash
# Set variables
WALLET="my-wallet"
CHAIN_ID="cosmoshub-4"
NODE="https://rpc.cosmos.network:443"
GAS_PRICES="0.025uatom"

# Upload wasm
TX_HASH=$(gaiad tx wasm store artifacts/payment_escrow.wasm \
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
# Instantiate with optional user registry
INIT_MSG='{"user_registry":null}'

gaiad tx wasm instantiate $CODE_ID "$INIT_MSG" \
  --from $WALLET \
  --label "Payment Escrow v1" \
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
# Query contract config
gaiad query wasm contract-state smart $CONTRACT_ADDRESS \
  '{"get_config":{}}' \
  --node $NODE
```

## Usage Examples

### Complete Payment Flow

#### 1. Receiver Creates Invoice

```bash
CONTRACT="<your-contract-address>"
RECEIVER_WALLET="receiver"

CREATE_MSG='{
  "create_invoice": {
    "invoice_id": "INV-2024-001",
    "amount": "5000000",
    "denom": "uatom",
    "reference": "Web development services - January 2024",
    "expires_in": 604800
  }
}'

gaiad tx wasm execute $CONTRACT "$CREATE_MSG" \
  --from $RECEIVER_WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

#### 2. Sender Funds Invoice

```bash
SENDER_WALLET="sender"

FUND_MSG='{"fund_invoice":{"invoice_id":"INV-2024-001"}}'

gaiad tx wasm execute $CONTRACT "$FUND_MSG" \
  --amount 5000000uatom \
  --from $SENDER_WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

#### 3. Receiver Releases Funds

```bash
RELEASE_MSG='{"release_funds":{"invoice_id":"INV-2024-001"}}'

gaiad tx wasm execute $CONTRACT "$RELEASE_MSG" \
  --from $RECEIVER_WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

### Query Invoice Status

```bash
gaiad query wasm contract-state smart $CONTRACT \
  '{"get_invoice":{"invoice_id":"INV-2024-001"}}' \
  --node $NODE
```

### List All Receiver Invoices

```bash
RECEIVER_ADDR=$(gaiad keys show $RECEIVER_WALLET -a)

gaiad query wasm contract-state smart $CONTRACT \
  '{"get_receiver_invoices":{"receiver":"'$RECEIVER_ADDR'","status":null}}' \
  --node $NODE
```

### Refund Scenario

If receiver needs to refund the sender:

```bash
REFUND_MSG='{"refund_funds":{"invoice_id":"INV-2024-001"}}'

gaiad tx wasm execute $CONTRACT "$REFUND_MSG" \
  --from $RECEIVER_WALLET \
  --chain-id $CHAIN_ID \
  --node $NODE \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices $GAS_PRICES -y
```

## Use Cases

### 1. Freelance Payments
```
Freelancer creates invoice → Client funds → Work completed → Freelancer releases
```

### 2. E-commerce
```
Seller creates invoice → Buyer funds → Goods shipped → Buyer confirms → Seller releases
```

### 3. Service Subscriptions
```
Service provider creates monthly invoice → Subscriber funds → Service delivered → Release
```

### 4. Dispute Resolution
```
Invoice funded → Dispute arises → Receiver refunds → Issue resolved externally
```

## Security Features

1. **Exact Amount Matching**: Sender must send exact invoice amount
2. **Receiver Control**: Only receiver can release or refund
3. **Single Funding**: Invoice can only be funded once
4. **Expiry Protection**: Expired invoices cannot be funded
5. **Status Validation**: State transitions are strictly enforced

## Error Handling

### Common Errors

**"Invoice not found"**
- The invoice ID doesn't exist. Check the ID and try again.

**"Already funded"**
- Invoice has already been funded. Cannot fund twice.

**"Incorrect amount"**
- Sent amount doesn't match invoice amount exactly.

**"Incorrect denom"**
- Sent wrong token denomination. Check invoice denom.

**"Expired"**
- Invoice has passed its expiration time.

**"Only receiver"**
- Only the invoice receiver can perform this action.

**"Not funded"**
- Cannot release/refund an invoice that hasn't been funded.

## Integration Example

### TypeScript/JavaScript

```typescript
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

// Create invoice
const createInvoiceMsg = {
  create_invoice: {
    invoice_id: "INV-2024-001",
    amount: "5000000",
    denom: "uatom",
    reference: "Payment for services",
    expires_in: 86400 // 24 hours
  }
};

await client.execute(
  receiverAddress,
  contractAddress,
  createInvoiceMsg,
  "auto"
);

// Fund invoice
const fundInvoiceMsg = {
  fund_invoice: {
    invoice_id: "INV-2024-001"
  }
};

await client.execute(
  senderAddress,
  contractAddress,
  fundInvoiceMsg,
  "auto",
  undefined,
  [{ denom: "uatom", amount: "5000000" }]
);

// Query invoice
const invoice = await client.queryContractSmart(
  contractAddress,
  { get_invoice: { invoice_id: "INV-2024-001" } }
);

console.log("Invoice status:", invoice.invoice.status);
```

## Best Practices

1. **Unique Invoice IDs**: Use timestamp or UUID-based IDs
2. **Clear References**: Include detailed payment descriptions
3. **Set Expiry**: Use reasonable expiration times (24-72 hours)
4. **Verify Before Release**: Confirm work/goods delivered before releasing
5. **Communication**: Coordinate with counterparty off-chain
6. **Test on Testnet**: Always test flows on testnet first

## Troubleshooting

### Gas Issues
```bash
# Increase gas limit
--gas 300000

# Or increase gas adjustment
--gas-adjustment 1.5
```

### Transaction Failed
```bash
# Check transaction details
gaiad query tx <TX_HASH> --node $NODE

# Check account balance
gaiad query bank balances $(gaiad keys show $WALLET -a) --node $NODE
```

### Query Not Working
```bash
# Verify contract address
gaiad query wasm contract $CONTRACT --node $NODE

# Try alternative RPC endpoint
NODE="https://cosmos-rpc.publicnode.com:443"
```

## Future Enhancements

- [ ] Multi-party escrow (arbitrator support)
- [ ] Partial payments
- [ ] Recurring invoices
- [ ] Invoice templates
- [ ] Dispute resolution mechanism
- [ ] Fee collection for platform
- [ ] Invoice notifications

## License

MIT
