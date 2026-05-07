use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Asset with denom already exists: {denom}")]
    DenomAlreadyExists { denom: String },

    #[error("Asset with ticker already exists: {ticker}")]
    TickerAlreadyExists { ticker: String },

    #[error("Asset not found: {denom}")]
    AssetNotFound { denom: String },

    #[error("Invalid decimals: {decimals}")]
    InvalidDecimals { decimals: u8 },

    #[error("Structured description key already exists: {key}")]
    StructuredDescriptionKeyExists { key: String },

    #[error("Structured description key not found: {key}")]
    StructuredDescriptionKeyNotFound { key: String },
}
