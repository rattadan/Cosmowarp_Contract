use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct UserPreferences {
    pub color: String,
    pub bio: Option<String>,
    pub dm_pubkey: Option<String>,
}

#[cw_serde]
pub struct ChatGroup {
    pub group_id: String,
    pub name: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub admin: String,
    pub is_public: bool,
    pub created_at: u64,
    pub message_count: u64,
}

#[cw_serde]
pub struct Message {
    pub message_id: u64,
    pub group_id: String,
    pub sender: String,
    pub content: String,
    pub sender_content: Option<String>,
    pub sender_dm_pubkey: Option<String>,
    pub timestamp: u64,
    pub modified: bool,
    pub modified_at: Option<u64>,
    pub reply_to: Option<u64>,
    pub thumbs_up: u64,
    pub thumbs_down: u64,
}

#[cw_serde]
pub enum VoteType {
    ThumbsUp,
    ThumbsDown,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const GROUPS: Map<&str, ChatGroup> = Map::new("groups");

pub const MESSAGES: Map<(&str, u64), Message> = Map::new("messages");

pub const MESSAGE_COUNTERS: Map<&str, u64> = Map::new("msg_counters");

pub const USER_GROUPS: Map<(&str, &str), ()> = Map::new("user_groups");

pub const USER_PREFERENCES: Map<&str, UserPreferences> = Map::new("user_prefs");

pub const MESSAGE_VOTES: Map<((&str, u64), &str), VoteType> = Map::new("msg_votes");
