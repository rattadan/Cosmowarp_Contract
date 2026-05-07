use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Profile not found for address: {address}")]
    ProfileNotFound { address: String },

    #[error("Invalid IPFS hash format")]
    InvalidIpfsHash {},
}
