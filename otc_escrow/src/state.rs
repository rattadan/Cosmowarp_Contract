use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::Map;

/// Vault status
#[cw_serde]
pub enum VaultStatus {
    /// Creator has funded, waiting for counterparty
    Open,
    /// Both sides funded, swap completed
    Completed,
    /// Vault cancelled by creator (before counterparty funded)
    Cancelled,
}

/// OTC Vault/Escrow data
#[cw_serde]
pub struct Vault {
    /// Unique vault ID
    pub vault_id: String,

    /// Creator's cosmos address (validated)
    pub creator: String,

    /// Counterparty's cosmos address (set when they fund)
    pub counterparty: Option<String>,

    /// Amount creator offers (their side)
    pub offer_amount: Uint128,

    /// Denom creator offers
    pub offer_denom: String,

    /// Amount creator wants in return (counterparty's side)
    pub ask_amount: Uint128,

    /// Denom creator wants
    pub ask_denom: String,

    /// Optional description/reference
    pub description: String,

    /// Current status
    pub status: VaultStatus,

    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,

    /// Creator funded timestamp
    pub creator_funded_at: Option<u64>,

    /// Completed timestamp (swap executed or cancelled)
    pub completed_at: Option<u64>,

    /// Optional expiry timestamp (0 = no expiry)
    pub expires_at: u64,
}

/// Primary store: Map from vault_id -> Vault.
pub const VAULTS: Map<&str, Vault> = Map::new("vaults");

/// Index of OPEN vaults for pagination. Key = vault_id.
/// Entries are added in `create_vault`, removed in `fund_vault`/`cancel_vault`.
pub const OPEN_INDEX: Map<&str, ()> = Map::new("open_idx");

/// Index of COMPLETED vaults sorted by completion time (descending iteration
/// returns most recent first). Key = (completed_at, vault_id).
pub const COMPLETED_INDEX: Map<(u64, &str), ()> = Map::new("done_idx");

/// Index of vaults by creator. Key = (creator, vault_id).
pub const CREATOR_INDEX: Map<(&str, &str), ()> = Map::new("creator_idx");

/// Index of vaults by counterparty. Key = (counterparty, vault_id).
pub const COUNTERPARTY_INDEX: Map<(&str, &str), ()> = Map::new("cp_idx");
