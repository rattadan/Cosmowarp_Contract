use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("DApp with ID already exists: {dapp_id}")]
    DAppAlreadyExists { dapp_id: String },

    #[error("DApp not found: {dapp_id}")]
    DAppNotFound { dapp_id: String },

    #[error("Trace data key already exists: {key}")]
    TraceDataKeyExists { key: String },

    #[error("Trace data key not found: {key}")]
    TraceDataKeyNotFound { key: String },

    // Star ranking system errors
    #[error("Insufficient stars: have {available}, need {required}")]
    InsufficientStars { available: u32, required: u32 },

    #[error("Invalid star amount: {stars}. Must be between 1 and 10")]
    InvalidStarAmount { stars: u32 },

    #[error("No stars assigned to dApp: {dapp_id}")]
    NoStarsAssigned { dapp_id: String },

    #[error("User star balance not found: {user}")]
    UserStarBalanceNotFound { user: String },

    #[error("Star assignment not found for user {user} and dApp {dapp_id}")]
    StarAssignmentNotFound { user: String, dapp_id: String },

    #[error("Cannot redeem more stars than assigned: have {assigned}, trying to redeem {trying}")]
    CannotRedeemMoreThanAssigned { assigned: u32, trying: u32 },
}
