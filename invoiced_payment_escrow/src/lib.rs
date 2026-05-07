//! Payment Escrow Contract for Interchain Pay
//! 
//! Handles P2P escrowed payments between two parties.
//! Receiver creates invoice, Sender funds escrow, Receiver releases or refunds.

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
