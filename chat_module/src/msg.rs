use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::VoteType;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateGroup {
        group_id: String,
        name: String,
        description: Option<String>,
        logo_url: Option<String>,
        is_public: bool,
    },

    DeleteGroup {
        group_id: String,
    },

    UpdateGroup {
        group_id: String,
        name: Option<String>,
        description: Option<String>,
        logo_url: Option<String>,
    },

    SendMessage {
        group_id: String,
        content: String,
        reply_to: Option<u64>,
    },

    EditMessage {
        group_id: String,
        message_id: u64,
        new_content: String,
    },

    DeleteMessage {
        group_id: String,
        message_id: u64,
    },

    SendDirectMessage {
        recipient: String,
        encrypted_content: String,
        sender_encrypted_content: String,
        sender_dm_pubkey: String,
    },

    UpdateConfig {
        admin: Option<String>,
    },

    SetUserColor {
        color: String,
    },

    SetUserPreferences {
        color: Option<String>,
        bio: Option<String>,
        dm_pubkey: Option<String>,
    },

    VoteMessage {
        group_id: String,
        message_id: u64,
        vote_type: VoteType,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},

    #[returns(Option<ChatGroup>)]
    GetGroup { group_id: String },

    #[returns(Vec<ChatGroup>)]
    ListPublicGroups {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Vec<ChatGroup>)]
    GetUserGroups {
        user: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Option<Message>)]
    GetMessage { group_id: String, message_id: u64 },

    #[returns(Vec<Message>)]
    ListMessages {
        group_id: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<Message>)]
    GetDirectMessages {
        counterparty: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Option<String>)]
    GetUserColor { user: String },

    #[returns(Option<UserPreferences>)]
    GetUserPreferences { user: String },

    #[returns(Option<String>)]
    GetUserDmPubKey { user: String },
}
