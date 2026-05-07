use crate::state::{Vault, VaultStatus};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new OTC vault (creator action).
    /// Creator defines what they offer and what they want in return.
    /// Must send exactly one coin (the offer) along with this message.
    CreateVault {
        /// Custom vault ID (must be unique, 1..=64 chars)
        vault_id: String,
        /// Amount creator wants in return
        ask_amount: Uint128,
        /// Denom creator wants (must be different from offer denom)
        ask_denom: String,
        /// Optional description (<=256 chars)
        description: String,
        /// Optional expiry in seconds from now (None / 0 = no expiry)
        expires_in: Option<u64>,
    },

    /// Fund the counterparty side of a vault (any user except creator).
    /// Must send exactly one coin matching (ask_denom, ask_amount).
    /// On success the swap executes atomically in the same transaction.
    FundVault { vault_id: String },

    /// Cancel a vault (creator only, only while still Open).
    /// Returns the creator's offered funds.
    CancelVault { vault_id: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get vault by ID.
    #[returns(VaultResponse)]
    GetVault { vault_id: String },

    /// Get open vaults (available for funding), paginated.
    #[returns(VaultsResponse)]
    GetOpenVaults {
        /// Optional: filter by offer_denom
        offer_denom: Option<String>,
        /// Optional: filter by ask_denom
        ask_denom: Option<String>,
        /// Optional: pagination cursor (vault_id to start AFTER)
        start_after: Option<String>,
        /// Optional: max results (1..=100, default 30)
        limit: Option<u32>,
    },

    /// Get completed vaults, most-recent first.
    #[returns(VaultsResponse)]
    GetCompletedVaults {
        /// Optional: max results (1..=100, default 30)
        limit: Option<u32>,
    },

    /// Get vaults created by an address.
    #[returns(VaultsResponse)]
    GetCreatorVaults {
        creator: String,
        status: Option<VaultStatus>,
        /// Optional: max results (1..=100, default 30)
        limit: Option<u32>,
    },

    /// Get vaults where address was counterparty.
    #[returns(VaultsResponse)]
    GetCounterpartyVaults {
        counterparty: String,
        /// Optional: max results (1..=100, default 30)
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct VaultResponse {
    pub vault: Option<Vault>,
}

#[cw_serde]
pub struct VaultsResponse {
    pub vaults: Vec<Vault>,
}
