use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{DAppEntry, Config};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddDApp {
        dapp_id: String,
        title: String,
        short_description: String,
        full_description: String,
        logo_url: String,
        banner_url: String,
        website: String,
        telegram: Option<String>,
        x: Option<String>,
        discord: Option<String>,
        github: Option<String>,
    },

    UpdateDApp {
        dapp_id: String,
        title: Option<String>,
        short_description: Option<String>,
        full_description: Option<String>,
        logo_url: Option<String>,
        banner_url: Option<String>,
        website: Option<String>,
        telegram: Option<String>,
        x: Option<String>,
        discord: Option<String>,
        github: Option<String>,
    },

    SetVerified {
        dapp_id: String,
        verified: bool,
    },

    SetBlocked {
        dapp_id: String,
        blocked: bool,
    },

    UpdateConfig {
        admin: Option<String>,
    },

    RemoveDApp {
        dapp_id: String,
    },

    AddTraceData {
        dapp_id: String,
        key: String,
        value: String,
    },

    UpdateTraceData {
        dapp_id: String,
        key: String,
        value: String,
    },

    RemoveTraceData {
        dapp_id: String,
        key: String,
    },
    // Star ranking system messages
    DistributeStars {
        dapp_id: String,
        stars: u32,
    },
    RedeemStars {
        dapp_id: String,
        stars: u32,
    },
    RedelegateStars {
        from_dapp_id: String,
        to_dapp_id: String,
        stars: u32,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<DAppEntry>)]
    GetDAppById { dapp_id: String },

    #[returns(Vec<DAppEntry>)]
    ListDApps {
        start_after: Option<String>,
        limit: Option<u32>,
        include_blocked: Option<bool>,
        only_verified: Option<bool>,
    },

    #[returns(Config)]
    GetConfig {},

    #[returns(String)]
    GetTraceData { dapp_id: String, key: String },

    #[returns(std::collections::HashMap<String, String>)]
    GetAllTraceData { dapp_id: String },

    // Star ranking system queries
    #[returns(UserStarBalance)]
    GetUserStarBalance { user: String },

    #[returns(Vec<StarAssignment>)]
    GetUserStarAssignments { user: String },

    #[returns(Vec<StarAssignment>)]
    GetDAppStarAssignments { dapp_id: String },

    #[returns(u32)]
    GetDAppTotalStars { dapp_id: String },
}
