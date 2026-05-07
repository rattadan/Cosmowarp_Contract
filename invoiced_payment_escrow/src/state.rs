use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

/// Invoice status
#[cw_serde]
pub enum InvoiceStatus {
    /// Invoice created, waiting for payment
    Pending,
    /// Payment received, funds in escrow
    Funded,
    /// Payment released to receiver
    Completed,
    /// Payment refunded to sender
    Refunded,
    /// Invoice cancelled by receiver before funding
    Cancelled,
}

/// Invoice/Escrow data
#[cw_serde]
pub struct Invoice {
    /// Unique invoice ID (e.g., "invoice_001")
    pub invoice_id: String,
    /// Receiver's cosmos address (creator of invoice)
    pub receiver: String,
    /// Sender's cosmos address (payer) - set when funded
    pub sender: Option<String>,
    /// Amount requested in base units
    pub amount: u128,
    /// Denom of requested funds (e.g., "uusdc" or "uatom")
    pub denom: String,
    /// Reference text for the invoice
    pub reference: String,
    /// Current status
    pub status: InvoiceStatus,
    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,
    /// Funded timestamp (seconds since epoch)
    pub funded_at: Option<u64>,
    /// Completed/Refunded timestamp
    pub completed_at: Option<u64>,
    /// Optional expiry timestamp (0 = no expiry)
    pub expires_at: u64,
}

/// Contract configuration
#[cw_serde]
pub struct Config {
    /// Optional: User registry contract address for profile lookups
    pub user_registry: Option<String>,
}

/// Map from invoice_id -> Invoice
pub const INVOICES: Map<&str, Invoice> = Map::new("invoices");

/// Map from receiver address -> list of invoice IDs
pub const RECEIVER_INVOICES: Map<&str, Vec<String>> = Map::new("receiver_invoices");

/// Map from sender address -> list of invoice IDs they funded
pub const SENDER_INVOICES: Map<&str, Vec<String>> = Map::new("sender_invoices");

/// Contract config
pub const CONFIG: Item<Config> = Item::new("config");

/// Invoice counter for generating unique IDs if needed
pub const INVOICE_COUNTER: Item<u64> = Item::new("invoice_counter");
