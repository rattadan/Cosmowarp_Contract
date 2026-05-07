use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{AssetEntry, Config};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddAsset {
        denom: String,
        name: String,
        ticker: String,
        image_url: String,
        description: String,
        website: Option<String>,
        x: Option<String>,
        discord: Option<String>,
        telegram: Option<String>,
        decimals: Option<u8>,
    },

    UpdateAsset {
        denom: String,
        name: Option<String>,
        ticker: Option<String>,
        image_url: Option<String>,
        description: Option<String>,
        website: Option<String>,
        x: Option<String>,
        discord: Option<String>,
        telegram: Option<String>,
        decimals: Option<u8>,
    },

    SetVerified {
        denom: String,
        verified: bool,
    },

    SetBlocked {
        denom: String,
        blocked: bool,
    },

    UpdateConfig {
        admin: Option<String>,
    },

    RemoveAsset {
        denom: String,
    },

    AddStructuredDescription {
        denom: String,
        key: String,
        value: String,
    },

    UpdateStructuredDescription {
        denom: String,
        key: String,
        value: String,
    },

    RemoveStructuredDescription {
        denom: String,
        key: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<AssetEntry>)]
    GetAssetByDenom { denom: String },

    #[returns(Option<AssetEntry>)]
    GetAssetByTicker { ticker: String },

    #[returns(Vec<AssetEntry>)]
    ListAssets {
        start_after: Option<String>,
        limit: Option<u32>,
        include_blocked: Option<bool>,
        only_verified: Option<bool>,
    },

    #[returns(Config)]
    GetConfig {},

    #[returns(Option<String>)]
    GetStructuredDescription { denom: String, key: String },

    #[returns(std::collections::HashMap<String, String>)]
    GetAllStructuredDescriptions { denom: String },
}
