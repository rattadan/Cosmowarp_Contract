use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use std::collections::HashMap;

#[cw_serde]
pub struct Config {
    pub admin: String,
}

#[cw_serde]
pub struct DAppEntry {
    pub dapp_id: String,
    pub title: String,
    pub short_description: String,
    pub full_description: String,
    pub logo_url: String,
    pub banner_url: String,
    pub website: String,
    pub telegram: Option<String>,
    pub x: Option<String>,
    pub discord: Option<String>,
    pub github: Option<String>,
    pub verified: bool,
    pub blocked: bool,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub trace_data: HashMap<String, String>,
    pub total_stars: u32,
}

#[cw_serde]
pub struct UserStarBalance {
    pub user: String,
    pub available_stars: u32,
    pub assigned_stars: u32,
}

#[cw_serde]
pub struct StarAssignment {
    pub user: String,
    pub dapp_id: String,
    pub stars: u32,
    pub assigned_at: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const DAPPS_BY_ID: Map<&str, DAppEntry> = Map::new("dapps_by_id");
pub const USER_STAR_BALANCES: Map<&str, UserStarBalance> = Map::new("user_star_balances");
pub const STAR_ASSIGNMENTS: Map<(&str, &str), StarAssignment> = Map::new("star_assignments");
pub const USER_STAR_ASSIGNMENTS: Map<&str, Vec<StarAssignment>> = Map::new("user_star_assignments");
pub const DAPP_STAR_ASSIGNMENTS: Map<&str, Vec<StarAssignment>> = Map::new("dapp_star_assignments");
