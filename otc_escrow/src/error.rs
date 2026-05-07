use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Vault not found: {vault_id}")]
    VaultNotFound { vault_id: String },

    #[error("Vault already exists: {vault_id}")]
    VaultAlreadyExists { vault_id: String },

    #[error("Vault not open for funding")]
    NotOpen {},

    #[error("Vault already completed or cancelled")]
    AlreadyCompleted {},

    #[error("Incorrect payment amount. Expected {expected}, got {received}")]
    IncorrectAmount { expected: String, received: String },

    #[error("Incorrect payment denom. Expected {expected}, got {received}")]
    IncorrectDenom { expected: String, received: String },

    #[error("Vault expired")]
    Expired {},

    #[error("Only creator can perform this action")]
    OnlyCreator {},

    #[error("Cannot swap same denom: {denom}")]
    SameDenom { denom: String },

    #[error("Invalid field length: {field} ({actual} not in {min}..={max})")]
    InvalidLength {
        field: String,
        actual: usize,
        min: usize,
        max: usize,
    },
}
