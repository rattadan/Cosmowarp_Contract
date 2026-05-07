use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invoice not found: {invoice_id}")]
    InvoiceNotFound { invoice_id: String },

    #[error("Invoice already funded")]
    AlreadyFunded {},

    #[error("Invoice not funded yet")]
    NotFunded {},

    #[error("Invoice already completed or cancelled")]
    AlreadyCompleted {},

    #[error("Incorrect payment amount. Expected {expected}, got {received}")]
    IncorrectAmount { expected: String, received: String },

    #[error("Incorrect payment denom. Expected {expected}, got {received}")]
    IncorrectDenom { expected: String, received: String },

    #[error("Invoice expired")]
    Expired {},

    #[error("Only receiver can perform this action")]
    OnlyReceiver {},

    #[error("Only sender can perform this action")]
    OnlySender {},
}
