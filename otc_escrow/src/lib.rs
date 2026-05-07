//! OTC Escrow Contract for Atomic Swaps
//! 
//! Handles P2P atomic swaps between two parties.
//! Creator defines a vault (e.g., 2 ATOM for 4 USDC), funds their side,
//! any counterparty can fund the other side, and upon both sides being
//! funded the swap executes atomically.

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
