#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ChatGroup, Config, Message, UserPreferences, VoteType, CONFIG, GROUPS, MESSAGES, MESSAGE_COUNTERS, USER_GROUPS, USER_PREFERENCES, MESSAGE_VOTES};

const CONTRACT_NAME: &str = "crates.io:chat-module";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 100;

fn is_admin(config: &Config, sender: &str) -> bool {
    match &config.admin {
        Some(admin) => admin == sender,
        None => false,
    }
}

fn assert_admin(config: &Config, sender: &str) -> Result<(), ContractError> {
    if is_admin(config, sender) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

fn generate_dm_group_id(addr1: &str, addr2: &str) -> String {
    let mut addrs = vec![addr1, addr2];
    addrs.sort();
    format!("dm:{}:{}", addrs[0], addrs[1])
}

fn is_dm_participant(group_id: &str, user: &str) -> bool {
    if !group_id.starts_with("dm:") {
        return false;
    }
    
    group_id.contains(user)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config { admin: msg.admin };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateGroup {
            group_id,
            name,
            description,
            logo_url,
            is_public,
        } => execute_create_group(deps, env, info, group_id, name, description, logo_url, is_public),

        ExecuteMsg::DeleteGroup { group_id } => execute_delete_group(deps, info, group_id),

        ExecuteMsg::UpdateGroup {
            group_id,
            name,
            description,
            logo_url,
        } => execute_update_group(deps, info, group_id, name, description, logo_url),

        ExecuteMsg::SendMessage {
            group_id,
            content,
            reply_to,
        } => execute_send_message(deps, env, info, group_id, content, reply_to),

        ExecuteMsg::EditMessage {
            group_id,
            message_id,
            new_content,
        } => execute_edit_message(deps, env, info, group_id, message_id, new_content),

        ExecuteMsg::DeleteMessage {
            group_id,
            message_id,
        } => execute_delete_message(deps, info, group_id, message_id),

        ExecuteMsg::SendDirectMessage {
            recipient,
            encrypted_content,
            sender_encrypted_content,
            sender_dm_pubkey,
        } => execute_send_direct_message(deps, env, info, recipient, encrypted_content, sender_encrypted_content, sender_dm_pubkey),

        ExecuteMsg::UpdateConfig { admin } => execute_update_config(deps, info, admin),

        ExecuteMsg::SetUserColor { color } => execute_set_user_color(deps, info, color),

        ExecuteMsg::SetUserPreferences { color, bio, dm_pubkey } => execute_set_user_preferences(deps, info, color, bio, dm_pubkey),

        ExecuteMsg::VoteMessage { group_id, message_id, vote_type } => execute_vote_message(deps, info, group_id, message_id, vote_type),
    }
}

fn execute_create_group(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    group_id: String,
    name: String,
    description: Option<String>,
    logo_url: Option<String>,
    is_public: bool,
) -> Result<Response, ContractError> {
    if GROUPS.has(deps.storage, &group_id) {
        return Err(ContractError::GroupAlreadyExists { group_id });
    }

    let group = ChatGroup {
        group_id: group_id.clone(),
        name: name.clone(),
        description,
        logo_url,
        admin: info.sender.to_string(),
        is_public,
        created_at: env.block.time.seconds(),
        message_count: 0,
    };

    GROUPS.save(deps.storage, &group_id, &group)?;
    USER_GROUPS.save(deps.storage, (&info.sender.to_string(), &group_id), &())?;
    MESSAGE_COUNTERS.save(deps.storage, &group_id, &0)?;

    Ok(Response::new()
        .add_attribute("method", "create_group")
        .add_attribute("group_id", group_id)
        .add_attribute("name", name)
        .add_attribute("admin", info.sender.to_string()))
}

fn execute_delete_group(
    deps: DepsMut,
    info: MessageInfo,
    group_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let group = GROUPS
        .may_load(deps.storage, &group_id)?
        .ok_or(ContractError::GroupNotFound { group_id: group_id.clone() })?;

    let is_contract_admin = is_admin(&config, &info.sender.to_string());
    let is_group_admin = group.admin == info.sender.to_string();
    let is_dm_participant = is_dm_participant(&group_id, &info.sender.to_string());

    if !is_contract_admin && !is_group_admin && !is_dm_participant {
        return Err(ContractError::Unauthorized {});
    }

    GROUPS.remove(deps.storage, &group_id);

    Ok(Response::new()
        .add_attribute("method", "delete_group")
        .add_attribute("group_id", group_id))
}

fn execute_update_group(
    deps: DepsMut,
    info: MessageInfo,
    group_id: String,
    name: Option<String>,
    description: Option<String>,
    logo_url: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut group = GROUPS
        .may_load(deps.storage, &group_id)?
        .ok_or(ContractError::GroupNotFound { group_id: group_id.clone() })?;

    let is_contract_admin = is_admin(&config, &info.sender.to_string());
    let is_group_admin = group.admin == info.sender.to_string();

    if !is_contract_admin && !is_group_admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(new_name) = name {
        group.name = new_name;
    }

    if let Some(new_description) = description {
        group.description = Some(new_description);
    }

    if let Some(new_logo_url) = logo_url {
        group.logo_url = Some(new_logo_url);
    }

    GROUPS.save(deps.storage, &group_id, &group)?;

    Ok(Response::new()
        .add_attribute("method", "update_group")
        .add_attribute("group_id", group_id))
}

fn execute_send_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    group_id: String,
    content: String,
    reply_to: Option<u64>,
) -> Result<Response, ContractError> {
    let mut group = GROUPS
        .may_load(deps.storage, &group_id)?
        .ok_or(ContractError::GroupNotFound { group_id: group_id.clone() })?;

    let message_id = MESSAGE_COUNTERS.load(deps.storage, &group_id)?;
    let new_message_id = message_id + 1;

    let message = Message {
        message_id: new_message_id,
        group_id: group_id.clone(),
        sender: info.sender.to_string(),
        content: content.clone(),
        sender_content: None,
        timestamp: env.block.time.seconds(),
        modified: false,
        modified_at: None,
        reply_to,
        thumbs_up: 0,
        thumbs_down: 0,
    };

    MESSAGES.save(deps.storage, (&group_id, new_message_id), &message)?;
    MESSAGE_COUNTERS.save(deps.storage, &group_id, &new_message_id)?;

    group.message_count += 1;
    GROUPS.save(deps.storage, &group_id, &group)?;

    USER_GROUPS.save(deps.storage, (&info.sender.to_string(), &group_id), &())?;

    Ok(Response::new()
        .add_attribute("method", "send_message")
        .add_attribute("group_id", group_id)
        .add_attribute("message_id", new_message_id.to_string())
        .add_attribute("sender", info.sender.to_string()))
}

fn execute_edit_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    group_id: String,
    message_id: u64,
    new_content: String,
) -> Result<Response, ContractError> {
    let mut message = MESSAGES
        .may_load(deps.storage, (&group_id, message_id))?
        .ok_or(ContractError::MessageNotFound { message_id })?;

    if message.sender != info.sender.to_string() {
        return Err(ContractError::CannotModifyOthersMessage {});
    }

    message.content = new_content;
    message.modified = true;
    message.modified_at = Some(env.block.time.seconds());

    MESSAGES.save(deps.storage, (&group_id, message_id), &message)?;

    Ok(Response::new()
        .add_attribute("method", "edit_message")
        .add_attribute("group_id", group_id)
        .add_attribute("message_id", message_id.to_string()))
}

fn execute_delete_message(
    deps: DepsMut,
    info: MessageInfo,
    group_id: String,
    message_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let group = GROUPS
        .may_load(deps.storage, &group_id)?
        .ok_or(ContractError::GroupNotFound { group_id: group_id.clone() })?;

    let message = MESSAGES
        .may_load(deps.storage, (&group_id, message_id))?
        .ok_or(ContractError::MessageNotFound { message_id })?;

    let is_contract_admin = is_admin(&config, &info.sender.to_string());
    let is_group_admin = group.admin == info.sender.to_string();
    let is_message_owner = message.sender == info.sender.to_string();

    if !is_contract_admin && !is_group_admin && !is_message_owner {
        return Err(ContractError::CannotDeleteMessage {});
    }

    MESSAGES.remove(deps.storage, (&group_id, message_id));

    Ok(Response::new()
        .add_attribute("method", "delete_message")
        .add_attribute("group_id", group_id)
        .add_attribute("message_id", message_id.to_string()))
}

fn execute_send_direct_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    encrypted_content: String,
    sender_encrypted_content: String,
    sender_dm_pubkey: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    let group_id = generate_dm_group_id(&sender, &recipient);

    if !GROUPS.has(deps.storage, &group_id) {
        let group = ChatGroup {
            group_id: group_id.clone(),
            name: format!("DM: {} <-> {}", sender, recipient),
            description: Some("Direct Message".to_string()),
            logo_url: None,
            admin: sender.clone(),
            is_public: false,
            created_at: env.block.time.seconds(),
            message_count: 0,
        };
        GROUPS.save(deps.storage, &group_id, &group)?;
        MESSAGE_COUNTERS.save(deps.storage, &group_id, &0)?;
    }

    let mut group = GROUPS.load(deps.storage, &group_id)?;
    let message_id = MESSAGE_COUNTERS.load(deps.storage, &group_id)?;
    let new_message_id = message_id + 1;

    let message = Message {
        message_id: new_message_id,
        group_id: group_id.clone(),
        sender: info.sender.to_string(),
        content: encrypted_content,
        sender_content: Some(sender_encrypted_content),
        sender_dm_pubkey: Some(sender_dm_pubkey),
        timestamp: env.block.time.seconds(),
        modified: false,
        modified_at: None,
        reply_to: None,
        thumbs_up: 0,
        thumbs_down: 0,
    };

    MESSAGES.save(deps.storage, (&group_id, new_message_id), &message)?;
    MESSAGE_COUNTERS.save(deps.storage, &group_id, &new_message_id)?;

    group.message_count += 1;
    GROUPS.save(deps.storage, &group_id, &group)?;

    USER_GROUPS.save(deps.storage, (&sender, &group_id), &())?;
    USER_GROUPS.save(deps.storage, (&recipient, &group_id), &())?;

    Ok(Response::new()
        .add_attribute("method", "send_direct_message")
        .add_attribute("group_id", group_id)
        .add_attribute("message_id", new_message_id.to_string())
        .add_attribute("sender", info.sender.to_string()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, &info.sender.to_string())?;

    let new_config = Config { admin };
    CONFIG.save(deps.storage, &new_config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

fn execute_set_user_color(
    deps: DepsMut,
    info: MessageInfo,
    color: String,
) -> Result<Response, ContractError> {
    let existing = USER_PREFERENCES.may_load(deps.storage, &info.sender.to_string())?;
    let bio = existing.and_then(|p| p.bio);
    
    let prefs = UserPreferences { 
        color: color.clone(),
        bio,
    };
    USER_PREFERENCES.save(deps.storage, &info.sender.to_string(), &prefs)?;

    Ok(Response::new()
        .add_attribute("method", "set_user_color")
        .add_attribute("user", info.sender.to_string())
        .add_attribute("color", color))
}

fn execute_set_user_preferences(
    deps: DepsMut,
    info: MessageInfo,
    color: Option<String>,
    bio: Option<String>,
    dm_pubkey: Option<String>,
) -> Result<Response, ContractError> {
    let existing = USER_PREFERENCES.may_load(deps.storage, &info.sender.to_string())?;
    
    let prefs = UserPreferences {
        color: color.unwrap_or_else(|| {
            existing.as_ref().map(|p| p.color.clone()).unwrap_or_else(|| "#667eea".to_string())
        }),
        bio: bio.or_else(|| existing.as_ref().and_then(|p| p.bio.clone())),
        dm_pubkey: dm_pubkey.or_else(|| existing.and_then(|p| p.dm_pubkey)),
    };
    
    USER_PREFERENCES.save(deps.storage, &info.sender.to_string(), &prefs)?;

    Ok(Response::new()
        .add_attribute("method", "set_user_preferences")
        .add_attribute("user", info.sender.to_string()))
}

fn execute_vote_message(
    deps: DepsMut,
    info: MessageInfo,
    group_id: String,
    message_id: u64,
    vote_type: VoteType,
) -> Result<Response, ContractError> {
    let mut message = MESSAGES
        .may_load(deps.storage, (&group_id, message_id))?
        .ok_or(ContractError::MessageNotFound { message_id })?;

    let voter = info.sender.to_string();
    
    let existing_vote = MESSAGE_VOTES.may_load(deps.storage, ((group_id.as_str(), message_id), voter.as_str()))?;
    
    if let Some(old_vote) = existing_vote {
        if old_vote == VoteType::ThumbsUp {
            message.thumbs_up = message.thumbs_up.saturating_sub(1);
        } else {
            message.thumbs_down = message.thumbs_down.saturating_sub(1);
        }
    }
    
    if vote_type == VoteType::ThumbsUp {
        message.thumbs_up += 1;
    } else {
        message.thumbs_down += 1;
    }
    
    MESSAGE_VOTES.save(deps.storage, ((group_id.as_str(), message_id), voter.as_str()), &vote_type)?;
    MESSAGES.save(deps.storage, (&group_id, message_id), &message)?;

    Ok(Response::new()
        .add_attribute("method", "vote_message")
        .add_attribute("group_id", group_id)
        .add_attribute("message_id", message_id.to_string())
        .add_attribute("vote_type", if vote_type == VoteType::ThumbsUp { "thumbs_up" } else { "thumbs_down" }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetGroup { group_id } => to_json_binary(&query_group(deps, group_id)?),
        QueryMsg::ListPublicGroups { start_after, limit } => {
            to_json_binary(&query_list_public_groups(deps, start_after, limit)?)
        }
        QueryMsg::GetUserGroups { user, start_after, limit } => {
            to_json_binary(&query_user_groups(deps, user, start_after, limit)?)
        }
        QueryMsg::GetMessage { group_id, message_id } => {
            to_json_binary(&query_message(deps, group_id, message_id)?)
        }
        QueryMsg::ListMessages { group_id, start_after, limit } => {
            to_json_binary(&query_list_messages(deps, group_id, start_after, limit)?)
        }
        QueryMsg::GetDirectMessages { counterparty, start_after, limit } => {
            to_json_binary(&query_direct_messages(deps, counterparty, start_after, limit)?)
        }
        QueryMsg::GetUserColor { user } => to_json_binary(&query_user_color(deps, user)?),
        QueryMsg::GetUserPreferences { user } => {
            to_json_binary(&query_user_preferences(deps, user)?)
        }
        QueryMsg::GetUserDmPubKey { user } => {
            to_json_binary(&query_user_dm_pubkey(deps, user)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_group(deps: Deps, group_id: String) -> StdResult<Option<ChatGroup>> {
    GROUPS.may_load(deps.storage, &group_id)
}

fn query_list_public_groups(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<ChatGroup>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|s| Bound::exclusive(s.as_str()));

    let groups: Vec<ChatGroup> = GROUPS
        .range(deps.storage, start, None, Order::Ascending)
        .filter_map(|item| {
            item.ok().and_then(|(_, group)| {
                if group.is_public {
                    Some(group)
                } else {
                    None
                }
            })
        })
        .take(limit)
        .collect();
    
    Ok(groups)
}

fn query_user_groups(
    deps: Deps,
    user: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<ChatGroup>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.as_ref().map(|s| Bound::exclusive(s.as_str()));

    let groups: Vec<ChatGroup> = USER_GROUPS
        .prefix(&user)
        .range(deps.storage, start, None, Order::Ascending)
        .filter_map(|item| {
            item.ok().and_then(|(group_id, _)| {
                GROUPS.may_load(deps.storage, &group_id).ok().flatten()
            })
        })
        .take(limit)
        .collect();
    
    Ok(groups)
}

fn query_message(deps: Deps, group_id: String, message_id: u64) -> StdResult<Option<Message>> {
    MESSAGES.may_load(deps.storage, (&group_id, message_id))
}

fn query_list_messages(
    deps: Deps,
    group_id: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Message>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|id| Bound::exclusive(id));

    MESSAGES
        .prefix(&group_id)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(_, msg)| msg))
        .collect()
}

fn query_direct_messages(
    deps: Deps,
    counterparty: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Message>> {
    let sender = counterparty.clone();
    let group_id = generate_dm_group_id(&sender, &counterparty);
    
    query_list_messages(deps, group_id, start_after, limit)
}

fn query_user_color(deps: Deps, user: String) -> StdResult<Option<String>> {
    let prefs = USER_PREFERENCES.may_load(deps.storage, &user)?;
    Ok(prefs.map(|p| p.color))
}

fn query_user_preferences(deps: Deps, user: String) -> StdResult<Option<UserPreferences>> {
    USER_PREFERENCES.may_load(deps.storage, &user)
}

fn query_user_dm_pubkey(deps: Deps, user: String) -> StdResult<Option<String>> {
    let prefs = USER_PREFERENCES.may_load(deps.storage, &user)?;
    Ok(prefs.and_then(|p| p.dm_pubkey))
}
