#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::{Bound, Map};
use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, DAppEntry, UserStarBalance, StarAssignment, CONFIG, DAPPS_BY_ID, 
    USER_STAR_BALANCES, STAR_ASSIGNMENTS, USER_STAR_ASSIGNMENTS, DAPP_STAR_ASSIGNMENTS
};

const CONTRACT_NAME: &str = "dapp-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INITIAL_USER_STARS: u32 = 10;

fn is_admin(config: &Config, sender: &str) -> bool {
    &config.admin == sender
}

fn assert_admin(config: &Config, sender: &str) -> Result<(), ContractError> {
    if is_admin(config, sender) {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config { admin: msg.admin.unwrap_or_else(|| info.sender.to_string()) };
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
        ExecuteMsg::AddDApp {
            dapp_id,
            title,
            short_description,
            full_description,
            logo_url,
            banner_url,
            website,
            telegram,
            x,
            discord,
            github,
        } => execute_add_dapp(
            deps,
            env,
            info,
            dapp_id,
            title,
            short_description,
            full_description,
            logo_url,
            banner_url,
            website,
            telegram,
            x,
            discord,
            github,
        ),

        ExecuteMsg::UpdateDApp {
            dapp_id,
            title,
            short_description,
            full_description,
            logo_url,
            banner_url,
            website,
            telegram,
            x,
            discord,
            github,
        } => execute_update_dapp(
            deps,
            env,
            info,
            dapp_id,
            title,
            short_description,
            full_description,
            logo_url,
            banner_url,
            website,
            telegram,
            x,
            discord,
            github,
        ),

        ExecuteMsg::SetVerified { dapp_id, verified } => {
            execute_set_verified(deps, env, info, dapp_id, verified)
        }

        ExecuteMsg::SetBlocked { dapp_id, blocked } => execute_set_blocked(deps, env, info, dapp_id, blocked),

        ExecuteMsg::UpdateConfig { admin } => execute_update_config(deps, info, admin),

        ExecuteMsg::RemoveDApp { dapp_id } => execute_remove_dapp(deps, env, info, dapp_id),

        ExecuteMsg::AddTraceData { dapp_id, key, value } => {
            execute_add_trace_data(deps, env, info, dapp_id, key, value)
        }

        ExecuteMsg::UpdateTraceData { dapp_id, key, value } => {
            execute_update_trace_data(deps, env, info, dapp_id, key, value)
        }

        ExecuteMsg::RemoveTraceData { dapp_id, key } => {
            execute_remove_trace_data(deps, env, info, dapp_id, key)
        }
        // Star ranking system messages
        ExecuteMsg::DistributeStars { dapp_id, stars } => {
            execute_distribute_stars(deps, env, info, dapp_id, stars)
        }
        ExecuteMsg::RedeemStars { dapp_id, stars } => {
            execute_redeem_stars(deps, env, info, dapp_id, stars)
        }
        ExecuteMsg::RedelegateStars {
            from_dapp_id,
            to_dapp_id,
            stars,
        } => execute_redelegate_stars(
            deps,
            env,
            info,
            from_dapp_id,
            to_dapp_id,
            stars,
        ),
    }
}

fn execute_add_dapp(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
) -> Result<Response, ContractError> {
    if DAPPS_BY_ID.has(deps.storage, &dapp_id) {
        return Err(ContractError::DAppAlreadyExists { dapp_id });
    }

    let now = env.block.time.seconds();
    let sender = info.sender.to_string();

    let entry = DAppEntry {
        dapp_id: dapp_id.clone(),
        title,
        short_description,
        full_description,
        logo_url,
        banner_url,
        website,
        telegram,
        x,
        discord,
        github,
        verified: false,
        blocked: false,
        created_by: sender.clone(),
        created_at: now,
        updated_at: now,
        trace_data: HashMap::new(),
        total_stars: 0,
    };

    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "add_dapp")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("created_by", sender))
}

fn execute_update_dapp(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(v) = title {
        entry.title = v;
    }
    if let Some(v) = short_description {
        entry.short_description = v;
    }
    if let Some(v) = full_description {
        entry.full_description = v;
    }
    if let Some(v) = logo_url {
        entry.logo_url = v;
    }
    if let Some(v) = banner_url {
        entry.banner_url = v;
    }
    if let Some(v) = website {
        entry.website = v;
    }
    if telegram.is_some() {
        entry.telegram = telegram;
    }
    if x.is_some() {
        entry.x = x;
    }
    if discord.is_some() {
        entry.discord = discord;
    }
    if github.is_some() {
        entry.github = github;
    }

    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "update_dapp")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("updated_by", sender))
}

fn execute_set_verified(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    verified: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    entry.verified = verified;
    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "set_verified")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("verified", verified.to_string()))
}

fn execute_set_blocked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    blocked: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    entry.blocked = blocked;
    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "set_blocked")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("blocked", blocked.to_string()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender.as_str() {
        return Err(ContractError::Unauthorized {});
    }

    config.admin = admin.unwrap_or(info.sender.to_string());
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

fn execute_remove_dapp(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    dapp_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    // Remove from storage
    DAPPS_BY_ID.remove(deps.storage, &dapp_id);

    Ok(Response::new()
        .add_attribute("method", "remove_dapp")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("title", entry.title)
        .add_attribute("removed_by", info.sender.to_string()))
}

fn execute_add_trace_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key already exists
    if entry.trace_data.contains_key(&key) {
        return Err(ContractError::TraceDataKeyExists { key });
    }

    entry.trace_data.insert(key.clone(), value);
    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "add_trace_data")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

fn execute_update_trace_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key exists
    if !entry.trace_data.contains_key(&key) {
        return Err(ContractError::TraceDataKeyNotFound { key });
    }

    entry.trace_data.insert(key.clone(), value);
    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "update_trace_data")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

fn execute_remove_trace_data(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    key: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = DAPPS_BY_ID
        .may_load(deps.storage, &dapp_id)?
        .ok_or(ContractError::DAppNotFound { dapp_id: dapp_id.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key exists and remove it
    if entry.trace_data.remove(&key).is_none() {
        return Err(ContractError::TraceDataKeyNotFound { key });
    }

    entry.updated_at = env.block.time.seconds();
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "remove_trace_data")
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDAppById { dapp_id } => to_json_binary(&query_dapp_by_id(deps, dapp_id)?),
        QueryMsg::ListDApps {
            start_after,
            limit,
            include_blocked,
            only_verified,
        } => to_json_binary(&query_list_dapps(
            deps,
            start_after,
            limit,
            include_blocked,
            only_verified,
        )?),
        QueryMsg::GetConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),

        QueryMsg::GetTraceData { dapp_id, key } => {
            to_json_binary(&query_trace_data(deps, dapp_id, key)?)
        }

        QueryMsg::GetAllTraceData { dapp_id } => {
            to_json_binary(&query_all_trace_data(deps, dapp_id)?)
        }
        // Star ranking system queries
        QueryMsg::GetUserStarBalance { user } => {
            to_json_binary(&query_user_star_balance(deps, user)?)
        }
        QueryMsg::GetUserStarAssignments { user } => {
            to_json_binary(&query_user_star_assignments(deps, user)?)
        }
        QueryMsg::GetDAppStarAssignments { dapp_id } => {
            to_json_binary(&query_dapp_star_assignments(deps, dapp_id)?)
        }
        QueryMsg::GetDAppTotalStars { dapp_id } => {
            to_json_binary(&query_dapp_total_stars(deps, dapp_id)?)
        }
    }
}

fn query_dapp_by_id(deps: Deps, dapp_id: String) -> StdResult<Option<DAppEntry>> {
    DAPPS_BY_ID.may_load(deps.storage, &dapp_id)
}

fn query_list_dapps(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    include_blocked: Option<bool>,
    only_verified: Option<bool>,
) -> StdResult<Vec<DAppEntry>> {
    let limit = limit.unwrap_or(100).min(500) as usize;
    let include_blocked = include_blocked.unwrap_or(false);
    let only_verified = only_verified.unwrap_or(false);

    let start = start_after.as_deref().map(Bound::exclusive);

    let mut out: Vec<DAppEntry> = Vec::new();

    for item in DAPPS_BY_ID
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit * 2)
    {
        let (_k, v) = item?;
        if !include_blocked && v.blocked {
            continue;
        }
        if only_verified && !v.verified {
            continue;
        }
        out.push(v);
        if out.len() >= limit {
            break;
        }
    }

    Ok(out)
}

fn query_trace_data(deps: Deps, dapp_id: String, key: String) -> StdResult<Option<String>> {
    if let Some(entry) = DAPPS_BY_ID.may_load(deps.storage, &dapp_id)? {
        Ok(entry.trace_data.get(&key).cloned())
    } else {
        Ok(None)
    }
}

fn query_all_trace_data(deps: Deps, dapp_id: String) -> StdResult<HashMap<String, String>> {
    if let Some(entry) = DAPPS_BY_ID.may_load(deps.storage, &dapp_id)? {
        Ok(entry.trace_data)
    } else {
        Ok(HashMap::new())
    }
}

// Star ranking system query functions

fn query_user_star_balance(deps: Deps, user: String) -> StdResult<UserStarBalance> {
    Ok(USER_STAR_BALANCES
        .may_load(deps.storage, &user)?
        .unwrap_or(UserStarBalance {
            user: user.clone(),
            available_stars: INITIAL_USER_STARS,
            assigned_stars: 0,
        }))
}

fn query_user_star_assignments(deps: Deps, user: String) -> StdResult<Vec<StarAssignment>> {
    Ok(USER_STAR_ASSIGNMENTS
        .may_load(deps.storage, &user)?
        .unwrap_or_default())
}

fn query_dapp_star_assignments(deps: Deps, dapp_id: String) -> StdResult<Vec<StarAssignment>> {
    Ok(DAPP_STAR_ASSIGNMENTS
        .may_load(deps.storage, &dapp_id)?
        .unwrap_or_default())
}

fn query_dapp_total_stars(deps: Deps, dapp_id: String) -> StdResult<u32> {
    if let Some(dapp) = DAPPS_BY_ID.may_load(deps.storage, &dapp_id)? {
        Ok(dapp.total_stars)
    } else {
        Ok(0)
    }
}

// Star ranking system functions

fn execute_distribute_stars(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    stars: u32,
) -> Result<Response, ContractError> {
    // Validate star amount
    if stars < 1 || stars > 10 {
        return Err(ContractError::InvalidStarAmount { stars });
    }

    // Check if dApp exists
    if !DAPPS_BY_ID.has(deps.storage, &dapp_id) {
        return Err(ContractError::DAppNotFound { dapp_id });
    }

    let sender = info.sender.to_string();
    let now = env.block.time.seconds();

    // Get or create user star balance
    let mut user_balance = USER_STAR_BALANCES
        .may_load(deps.storage, &sender)?
        .unwrap_or(UserStarBalance {
            user: sender.clone(),
            available_stars: INITIAL_USER_STARS,
            assigned_stars: 0,
        });

    // Check if user has enough stars
    if user_balance.available_stars < stars {
        return Err(ContractError::InsufficientStars {
            available: user_balance.available_stars,
            required: stars,
        });
    }

    // Update user balance
    user_balance.available_stars -= stars;
    user_balance.assigned_stars += stars;
    USER_STAR_BALANCES.save(deps.storage, &sender, &user_balance)?;

    // Create or update star assignment
    let assignment_key = (&sender[..], &dapp_id[..]);
    let mut star_assignment = STAR_ASSIGNMENTS
        .may_load(deps.storage, assignment_key)?
        .unwrap_or(StarAssignment {
            user: sender.clone(),
            dapp_id: dapp_id.clone(),
            stars: 0,
            assigned_at: now,
        });

    star_assignment.stars += stars;
    star_assignment.assigned_at = now;
    STAR_ASSIGNMENTS.save(deps.storage, assignment_key, &star_assignment)?;

    // Update user assignments list
    let mut user_assignments = USER_STAR_ASSIGNMENTS
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();
    
    // Remove existing assignment for this dApp if it exists
    user_assignments.retain(|a| a.dapp_id != dapp_id);
    user_assignments.push(star_assignment.clone());
    USER_STAR_ASSIGNMENTS.save(deps.storage, &sender, &user_assignments)?;

    // Update dApp assignments list
    let mut dapp_assignments = DAPP_STAR_ASSIGNMENTS
        .may_load(deps.storage, &dapp_id)?
        .unwrap_or_default();
    
    // Remove existing assignment for this user if it exists
    dapp_assignments.retain(|a| a.user != sender);
    dapp_assignments.push(star_assignment.clone());
    DAPP_STAR_ASSIGNMENTS.save(deps.storage, &dapp_id, &dapp_assignments)?;

    // Update dApp total stars
    let mut dapp = DAPPS_BY_ID.load(deps.storage, &dapp_id)?;
    dapp.total_stars += stars;
    dapp.updated_at = now;
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &dapp)?;

    Ok(Response::new()
        .add_attribute("method", "distribute_stars")
        .add_attribute("user", sender)
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("stars", stars.to_string()))
}

fn execute_redeem_stars(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    dapp_id: String,
    stars: u32,
) -> Result<Response, ContractError> {
    // Validate star amount
    if stars < 1 || stars > 10 {
        return Err(ContractError::InvalidStarAmount { stars });
    }

    let sender = info.sender.to_string();
    let now = env.block.time.seconds();

    // Check if star assignment exists
    let assignment_key = (&sender[..], &dapp_id[..]);
    let mut star_assignment = STAR_ASSIGNMENTS
        .may_load(deps.storage, assignment_key)?
        .ok_or(ContractError::StarAssignmentNotFound {
            user: sender.clone(),
            dapp_id: dapp_id.clone(),
        })?;

    // Check if trying to redeem more than assigned
    if star_assignment.stars < stars {
        return Err(ContractError::CannotRedeemMoreThanAssigned {
            assigned: star_assignment.stars,
            trying: stars,
        });
    }

    // Update star assignment
    star_assignment.stars -= stars;
    star_assignment.assigned_at = now;

    if star_assignment.stars == 0 {
        // Remove assignment if no stars left
        STAR_ASSIGNMENTS.remove(deps.storage, assignment_key);
        
        // Remove from user assignments list
        let mut user_assignments = USER_STAR_ASSIGNMENTS
            .may_load(deps.storage, &sender)?
            .unwrap_or_default();
        user_assignments.retain(|a| a.dapp_id != dapp_id);
        if user_assignments.is_empty() {
            USER_STAR_ASSIGNMENTS.remove(deps.storage, &sender);
        } else {
            USER_STAR_ASSIGNMENTS.save(deps.storage, &sender, &user_assignments)?;
        }
        
        // Remove from dApp assignments list
        let mut dapp_assignments = DAPP_STAR_ASSIGNMENTS
            .may_load(deps.storage, &dapp_id)?
            .unwrap_or_default();
        dapp_assignments.retain(|a| a.user != sender);
        if dapp_assignments.is_empty() {
            DAPP_STAR_ASSIGNMENTS.remove(deps.storage, &dapp_id);
        } else {
            DAPP_STAR_ASSIGNMENTS.save(deps.storage, &dapp_id, &dapp_assignments)?;
        }
    } else {
        // Update assignment
        STAR_ASSIGNMENTS.save(deps.storage, assignment_key, &star_assignment)?;
        
        // Update in user assignments list
        let mut user_assignments = USER_STAR_ASSIGNMENTS
            .may_load(deps.storage, &sender)?
            .unwrap_or_default();
        if let Some(index) = user_assignments.iter().position(|a| a.dapp_id == dapp_id) {
            user_assignments[index] = star_assignment.clone();
        }
        USER_STAR_ASSIGNMENTS.save(deps.storage, &sender, &user_assignments)?;
        
        // Update in dApp assignments list
        let mut dapp_assignments = DAPP_STAR_ASSIGNMENTS
            .may_load(deps.storage, &dapp_id)?
            .unwrap_or_default();
        if let Some(index) = dapp_assignments.iter().position(|a| a.user == sender) {
            dapp_assignments[index] = star_assignment.clone();
        }
        DAPP_STAR_ASSIGNMENTS.save(deps.storage, &dapp_id, &dapp_assignments)?;
    }

    // Update user balance
    let mut user_balance = USER_STAR_BALANCES
        .may_load(deps.storage, &sender)?
        .ok_or(ContractError::UserStarBalanceNotFound {
            user: sender.clone(),
        })?;
    user_balance.available_stars += stars;
    user_balance.assigned_stars -= stars;
    USER_STAR_BALANCES.save(deps.storage, &sender, &user_balance)?;

    // Update dApp total stars
    let mut dapp = DAPPS_BY_ID.load(deps.storage, &dapp_id)?;
    dapp.total_stars = dapp.total_stars.saturating_sub(stars);
    dapp.updated_at = now;
    DAPPS_BY_ID.save(deps.storage, &dapp_id, &dapp)?;

    Ok(Response::new()
        .add_attribute("method", "redeem_stars")
        .add_attribute("user", sender)
        .add_attribute("dapp_id", dapp_id)
        .add_attribute("stars", stars.to_string()))
}

fn execute_redelegate_stars(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from_dapp_id: String,
    to_dapp_id: String,
    stars: u32,
) -> Result<Response, ContractError> {
    // Validate star amount
    if stars < 1 || stars > 10 {
        return Err(ContractError::InvalidStarAmount { stars });
    }

    // Check if both dApps exist
    if !DAPPS_BY_ID.has(deps.storage, &from_dapp_id) {
        return Err(ContractError::DAppNotFound {
            dapp_id: from_dapp_id,
        });
    }
    if !DAPPS_BY_ID.has(deps.storage, &to_dapp_id) {
        return Err(ContractError::DAppNotFound {
            dapp_id: to_dapp_id,
        });
    }

    let sender = info.sender.to_string();
    let now = env.block.time.seconds();

    // Check if star assignment exists for from_dapp
    let assignment_key = (&sender[..], &from_dapp_id[..]);
    let mut from_assignment = STAR_ASSIGNMENTS
        .may_load(deps.storage, assignment_key)?
        .ok_or(ContractError::StarAssignmentNotFound {
            user: sender.clone(),
            dapp_id: from_dapp_id.clone(),
        })?;

    // Check if trying to redelegate more than assigned
    if from_assignment.stars < stars {
        return Err(ContractError::CannotRedeemMoreThanAssigned {
            assigned: from_assignment.stars,
            trying: stars,
        });
    }

    // Remove stars from from_dapp
    from_assignment.stars -= stars;
    if from_assignment.stars == 0 {
        STAR_ASSIGNMENTS.remove(deps.storage, assignment_key);
    } else {
        from_assignment.assigned_at = now;
        STAR_ASSIGNMENTS.save(deps.storage, assignment_key, &from_assignment)?;
    }

    // Add stars to to_dapp
    let to_assignment_key = (&sender[..], &to_dapp_id[..]);
    let mut to_assignment = STAR_ASSIGNMENTS
        .may_load(deps.storage, to_assignment_key)?
        .unwrap_or(StarAssignment {
            user: sender.clone(),
            dapp_id: to_dapp_id.clone(),
            stars: 0,
            assigned_at: now,
        });
    to_assignment.stars += stars;
    to_assignment.assigned_at = now;
    STAR_ASSIGNMENTS.save(deps.storage, to_assignment_key, &to_assignment)?;

    // Update user assignments list
    let mut user_assignments = USER_STAR_ASSIGNMENTS
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();
    
    // Remove or update from_dapp assignment
    user_assignments.retain(|a| a.dapp_id != from_dapp_id);
    if from_assignment.stars > 0 {
        user_assignments.push(from_assignment.clone());
    }
    
    // Remove or update to_dapp assignment
    user_assignments.retain(|a| a.dapp_id != to_dapp_id);
    user_assignments.push(to_assignment.clone());
    
    USER_STAR_ASSIGNMENTS.save(deps.storage, &sender, &user_assignments)?;

    // Update dApp assignments for from_dapp
    let mut from_dapp_assignments = DAPP_STAR_ASSIGNMENTS
        .may_load(deps.storage, &from_dapp_id)?
        .unwrap_or_default();
    from_dapp_assignments.retain(|a| a.user != sender);
    if from_assignment.stars > 0 {
        from_dapp_assignments.push(from_assignment);
    }
    if from_dapp_assignments.is_empty() {
        DAPP_STAR_ASSIGNMENTS.remove(deps.storage, &from_dapp_id);
    } else {
        DAPP_STAR_ASSIGNMENTS.save(deps.storage, &from_dapp_id, &from_dapp_assignments)?;
    }

    // Update dApp assignments for to_dapp
    let mut to_dapp_assignments = DAPP_STAR_ASSIGNMENTS
        .may_load(deps.storage, &to_dapp_id)?
        .unwrap_or_default();
    to_dapp_assignments.retain(|a| a.user != sender);
    to_dapp_assignments.push(to_assignment.clone());
    DAPP_STAR_ASSIGNMENTS.save(deps.storage, &to_dapp_id, &to_dapp_assignments)?;

    // Update dApp total stars
    let mut from_dapp = DAPPS_BY_ID.load(deps.storage, &from_dapp_id)?;
    from_dapp.total_stars = from_dapp.total_stars.saturating_sub(stars);
    from_dapp.updated_at = now;
    DAPPS_BY_ID.save(deps.storage, &from_dapp_id, &from_dapp)?;

    let mut to_dapp = DAPPS_BY_ID.load(deps.storage, &to_dapp_id)?;
    to_dapp.total_stars += stars;
    to_dapp.updated_at = now;
    DAPPS_BY_ID.save(deps.storage, &to_dapp_id, &to_dapp)?;

    Ok(Response::new()
        .add_attribute("method", "redelegate_stars")
        .add_attribute("user", sender)
        .add_attribute("from_dapp_id", from_dapp_id)
        .add_attribute("to_dapp_id", to_dapp_id)
        .add_attribute("stars", stars.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn add_dapp_enforces_uniqueness() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("any", &[]),
            InstantiateMsg {
                admin: Some("admin".to_string()),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::AddDApp {
                dapp_id: "my-dapp".to_string(),
                title: "My DApp".to_string(),
                short_description: "A short description".to_string(),
                full_description: "A full description".to_string(),
                logo_url: "https://example.com/logo.png".to_string(),
                banner_url: "https://example.com/banner.png".to_string(),
                website: "https://example.com".to_string(),
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap();

        // Same dapp_id should fail
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddDApp {
                dapp_id: "my-dapp".to_string(),
                title: "Another DApp".to_string(),
                short_description: "Another description".to_string(),
                full_description: "Another full description".to_string(),
                logo_url: "https://example.com/logo2.png".to_string(),
                banner_url: "https://example.com/banner2.png".to_string(),
                website: "https://example2.com".to_string(),
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::DAppAlreadyExists { .. }));
    }

    #[test]
    fn only_creator_or_admin_can_update() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("any", &[]),
            InstantiateMsg {
                admin: Some("admin".to_string()),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::AddDApp {
                dapp_id: "test-dapp".to_string(),
                title: "Test DApp".to_string(),
                short_description: "Test short description".to_string(),
                full_description: "Test full description".to_string(),
                logo_url: "https://example.com/logo.png".to_string(),
                banner_url: "https://example.com/banner.png".to_string(),
                website: "https://example.com".to_string(),
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap();

        // random user cannot update
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::UpdateDApp {
                dapp_id: "test-dapp".to_string(),
                title: None,
                short_description: None,
                full_description: None,
                logo_url: Some("https://example.com/new-logo.png".to_string()),
                banner_url: None,
                website: None,
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized { .. }));

        // creator can update
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateDApp {
                dapp_id: "test-dapp".to_string(),
                title: Some("Updated Title".to_string()),
                short_description: Some("Updated short description".to_string()),
                full_description: Some("Updated full description".to_string()),
                logo_url: Some("https://example.com/new-logo.png".to_string()),
                banner_url: Some("https://example.com/new-banner.png".to_string()),
                website: Some("https://updated-example.com".to_string()),
                telegram: Some("https://t.me/test".to_string()),
                x: Some("https://x.com/test".to_string()),
                discord: Some("https://discord.gg/test".to_string()),
                github: Some("https://github.com/test".to_string()),
            },
        )
        .unwrap();

        // admin can update
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateDApp {
                dapp_id: "test-dapp".to_string(),
                title: Some("Admin Title".to_string()),
                short_description: None,
                full_description: Some("Admin override".to_string()),
                logo_url: None,
                banner_url: None,
                website: None,
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn admin_only_verified_and_blocked() {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("any", &[]),
            InstantiateMsg {
                admin: Some("admin".to_string()),
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::AddDApp {
                dapp_id: "test-dapp".to_string(),
                title: "Test DApp".to_string(),
                short_description: "Test short description".to_string(),
                full_description: "Test full description".to_string(),
                logo_url: "https://example.com/logo.png".to_string(),
                banner_url: "https://example.com/banner.png".to_string(),
                website: "https://example.com".to_string(),
                telegram: None,
                x: None,
                discord: None,
                github: None,
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetVerified {
                dapp_id: "test-dapp".to_string(),
                verified: true,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized { .. }));

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::SetVerified {
                dapp_id: "test-dapp".to_string(),
                verified: true,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::SetBlocked {
                dapp_id: "test-dapp".to_string(),
                blocked: true,
            },
        )
        .unwrap();

        let dapp = query_dapp_by_id(deps.as_ref(), "test-dapp".to_string())
            .unwrap()
            .unwrap();
        assert!(dapp.verified);
        assert!(dapp.blocked);
    }
}
