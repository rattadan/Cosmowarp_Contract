use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::{Invoice, InvoiceStatus};

#[cw_serde]
pub struct InstantiateMsg {
    /// Optional: User registry contract address
    pub user_registry: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new invoice (receiver action)
    CreateInvoice {
        /// Custom invoice ID (must be unique)
        invoice_id: String,
        /// Amount in base units (e.g., 1000000 for 1 USDC)
        amount: u128,
        /// Denom: "uusdc" for USDC or "uatom" for ATOM
        denom: String,
        /// Reference text (e.g., "Order #123")
        reference: String,
        /// Optional expiry in seconds from now (0 = no expiry)
        expires_in: Option<u64>,
    },
    
    /// Fund an invoice (sender action)
    /// Must send exact amount in the correct denom
    FundInvoice {
        invoice_id: String,
    },
    
    /// Release funds to receiver (receiver confirms goods/services delivered)
    /// Can be called by receiver or automatically after timeout
    ReleaseFunds {
        invoice_id: String,
    },
    
    /// Refund funds to sender (receiver cancels or dispute resolved)
    RefundFunds {
        invoice_id: String,
    },
    
    /// Cancel unfunded invoice (receiver only)
    CancelInvoice {
        invoice_id: String,
    },
    
    /// Update user registry address (admin only - contract instantiator)
    UpdateConfig {
        user_registry: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get invoice by ID
    #[returns(InvoiceResponse)]
    GetInvoice { invoice_id: String },
    
    /// Get all invoices for a receiver
    #[returns(InvoicesResponse)]
    GetReceiverInvoices { 
        receiver: String,
        status: Option<InvoiceStatus>,
    },
    
    /// Get all invoices funded by a sender
    #[returns(InvoicesResponse)]
    GetSenderInvoices { 
        sender: String,
        status: Option<InvoiceStatus>,
    },
    
    /// Get contract config
    #[returns(ConfigResponse)]
    GetConfig {},
}

#[cw_serde]
pub struct InvoiceResponse {
    pub invoice: Option<Invoice>,
}

#[cw_serde]
pub struct InvoicesResponse {
    pub invoices: Vec<Invoice>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub user_registry: Option<String>,
}
