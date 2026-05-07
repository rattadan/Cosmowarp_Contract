use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use std::collections::HashMap;

#[cw_serde]
pub struct Config {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct AssetEntry {
    pub denom: String,
    pub name: String,
    pub ticker: String,
    pub image_url: String,
    pub description: String,
    pub website: Option<String>,
    pub x: Option<String>,
    pub discord: Option<String>,
    pub telegram: Option<String>,
    pub decimals: u8,
    pub verified: bool,
    pub blocked: bool,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    // New structured descriptions for extensibility
    pub structured_descriptions: HashMap<String, String>,
}

pub const CONFIG: Item<Config> = Item::new("config");

/// Primary storage: denom -> entry
pub const ASSETS_BY_DENOM: Map<&str, AssetEntry> = Map::new("assets_by_denom");

/// Secondary index: ticker (normalized uppercase) -> denom
pub const DENOM_BY_TICKER: Map<&str, String> = Map::new("denom_by_ticker");
