//! User Registry Contract for Interchain Pay
//! 
//! Stores optional user profile data linked to Cosmos addresses.
//! All fields except cosmos_address are optional.

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
