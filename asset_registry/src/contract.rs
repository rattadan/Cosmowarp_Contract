#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use std::collections::HashMap;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{AssetEntry, Config, ASSETS_BY_DENOM, CONFIG, DENOM_BY_TICKER};

const CONTRACT_NAME: &str = "crates.io:asset-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_DECIMALS: u8 = 6;
const MAX_DECIMALS: u8 = 18;

fn normalize_ticker(ticker: &str) -> String {
    ticker.trim().to_ascii_uppercase()
}

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

fn validate_decimals(decimals: u8) -> Result<(), ContractError> {
    if decimals > MAX_DECIMALS {
        return Err(ContractError::InvalidDecimals { decimals });
    }
    Ok(())
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
        ExecuteMsg::AddAsset {
            denom,
            name,
            ticker,
            image_url,
            description,
            website,
            x,
            discord,
            telegram,
            decimals,
        } => execute_add_asset(
            deps,
            env,
            info,
            denom,
            name,
            ticker,
            image_url,
            description,
            website,
            x,
            discord,
            telegram,
            decimals,
        ),

        ExecuteMsg::UpdateAsset {
            denom,
            name,
            ticker,
            image_url,
            description,
            website,
            x,
            discord,
            telegram,
            decimals,
        } => execute_update_asset(
            deps,
            env,
            info,
            denom,
            name,
            ticker,
            image_url,
            description,
            website,
            x,
            discord,
            telegram,
            decimals,
        ),

        ExecuteMsg::SetVerified { denom, verified } => {
            execute_set_verified(deps, env, info, denom, verified)
        }

        ExecuteMsg::SetBlocked { denom, blocked } => execute_set_blocked(deps, env, info, denom, blocked),

        ExecuteMsg::UpdateConfig { admin } => execute_update_config(deps, info, admin),

        ExecuteMsg::RemoveAsset { denom } => execute_remove_asset(deps, env, info, denom),

        ExecuteMsg::AddStructuredDescription { denom, key, value } => {
            execute_add_structured_description(deps, env, info, denom, key, value)
        }

        ExecuteMsg::UpdateStructuredDescription { denom, key, value } => {
            execute_update_structured_description(deps, env, info, denom, key, value)
        }

        ExecuteMsg::RemoveStructuredDescription { denom, key } => {
            execute_remove_structured_description(deps, env, info, denom, key)
        }
    }
}

fn execute_add_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
) -> Result<Response, ContractError> {
    if ASSETS_BY_DENOM.has(deps.storage, &denom) {
        return Err(ContractError::DenomAlreadyExists { denom });
    }

    let ticker_norm = normalize_ticker(&ticker);
    if DENOM_BY_TICKER.has(deps.storage, &ticker_norm) {
        return Err(ContractError::TickerAlreadyExists {
            ticker: ticker_norm,
        });
    }

    let decimals_final = decimals.unwrap_or(DEFAULT_DECIMALS);
    validate_decimals(decimals_final)?;

    let now = env.block.time.seconds();
    let sender = info.sender.to_string();

    let entry = AssetEntry {
        denom: denom.clone(),
        name,
        ticker: ticker_norm.clone(),
        image_url,
        description,
        website,
        x,
        discord,
        telegram,
        decimals: decimals_final,
        verified: false,
        blocked: false,
        created_by: sender.clone(),
        created_at: now,
        updated_at: now,
        structured_descriptions: HashMap::new(),
    };

    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;
    DENOM_BY_TICKER.save(deps.storage, &ticker_norm, &denom)?;

    Ok(Response::new()
        .add_attribute("method", "add_asset")
        .add_attribute("denom", denom)
        .add_attribute("ticker", ticker_norm)
        .add_attribute("created_by", sender))
}

fn execute_update_asset(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Handle ticker update - only admin can change ticker
    if let Some(new_ticker) = ticker {
        let sender_is_admin = is_admin(&config, &sender);
        if !sender_is_admin {
            return Err(ContractError::Unauthorized {});
        }
        
        let new_ticker_norm = normalize_ticker(&new_ticker);
        
        // Check if new ticker already exists (and belongs to different denom)
        if let Some(existing_denom) = DENOM_BY_TICKER.may_load(deps.storage, &new_ticker_norm)? {
            if existing_denom != denom {
                return Err(ContractError::TickerAlreadyExists {
                    ticker: new_ticker_norm,
                });
            }
        }
        
        // Remove old ticker index and add new one
        DENOM_BY_TICKER.remove(deps.storage, &entry.ticker);
        entry.ticker = new_ticker_norm.clone();
        DENOM_BY_TICKER.save(deps.storage, &new_ticker_norm, &denom)?;
    }

    if let Some(v) = image_url {
        entry.image_url = v;
    }
    if let Some(v) = description {
        entry.description = v;
    }
    if let Some(v) = name {
        entry.name = v;
    }
    if website.is_some() {
        entry.website = website;
    }
    if x.is_some() {
        entry.x = x;
    }
    if discord.is_some() {
        entry.discord = discord;
    }
    if telegram.is_some() {
        entry.telegram = telegram;
    }
    if let Some(v) = decimals {
        validate_decimals(v)?;
        entry.decimals = v;
    }

    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "update_asset")
        .add_attribute("denom", denom)
        .add_attribute("updated_by", sender))
}

fn execute_set_verified(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    verified: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    entry.verified = verified;
    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "set_verified")
        .add_attribute("denom", denom)
        .add_attribute("verified", verified.to_string()))
}

fn execute_set_blocked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    blocked: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    entry.blocked = blocked;
    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "set_blocked")
        .add_attribute("denom", denom)
        .add_attribute("blocked", blocked.to_string()))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if let Some(ref current_admin) = config.admin {
        if info.sender.to_string() != *current_admin {
            return Err(ContractError::Unauthorized {});
        }
    }

    config.admin = admin;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

fn execute_remove_asset(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    assert_admin(&config, info.sender.as_str())?;

    let entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    // Remove from both storage maps
    ASSETS_BY_DENOM.remove(deps.storage, &denom);
    DENOM_BY_TICKER.remove(deps.storage, &entry.ticker);

    Ok(Response::new()
        .add_attribute("method", "remove_asset")
        .add_attribute("denom", denom)
        .add_attribute("ticker", entry.ticker)
        .add_attribute("removed_by", info.sender.to_string()))
}

fn execute_add_structured_description(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key already exists
    if entry.structured_descriptions.contains_key(&key) {
        return Err(ContractError::StructuredDescriptionKeyExists { key });
    }

    entry.structured_descriptions.insert(key.clone(), value);
    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "add_structured_description")
        .add_attribute("denom", denom)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

fn execute_update_structured_description(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key exists
    if !entry.structured_descriptions.contains_key(&key) {
        return Err(ContractError::StructuredDescriptionKeyNotFound { key });
    }

    entry.structured_descriptions.insert(key.clone(), value);
    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "update_structured_description")
        .add_attribute("denom", denom)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

fn execute_remove_structured_description(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: String,
    key: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    let mut entry = ASSETS_BY_DENOM
        .may_load(deps.storage, &denom)?
        .ok_or(ContractError::AssetNotFound { denom: denom.clone() })?;

    let sender = info.sender.to_string();
    let allowed = sender == entry.created_by || is_admin(&config, &sender);
    if !allowed {
        return Err(ContractError::Unauthorized {});
    }

    // Check if key exists and remove it
    if entry.structured_descriptions.remove(&key).is_none() {
        return Err(ContractError::StructuredDescriptionKeyNotFound { key });
    }

    entry.updated_at = env.block.time.seconds();
    ASSETS_BY_DENOM.save(deps.storage, &denom, &entry)?;

    Ok(Response::new()
        .add_attribute("method", "remove_structured_description")
        .add_attribute("denom", denom)
        .add_attribute("key", key)
        .add_attribute("updated_by", sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAssetByDenom { denom } => to_json_binary(&query_asset_by_denom(deps, denom)?),
        QueryMsg::GetAssetByTicker { ticker } => {
            to_json_binary(&query_asset_by_ticker(deps, ticker)?)
        }
        QueryMsg::ListAssets {
            start_after,
            limit,
            include_blocked,
            only_verified,
        } => to_json_binary(&query_list_assets(
            deps,
            start_after,
            limit,
            include_blocked,
            only_verified,
        )?),
        QueryMsg::GetConfig {} => to_json_binary(&CONFIG.load(deps.storage)?),

        QueryMsg::GetStructuredDescription { denom, key } => {
            to_json_binary(&query_structured_description(deps, denom, key)?)
        }

        QueryMsg::GetAllStructuredDescriptions { denom } => {
            to_json_binary(&query_all_structured_descriptions(deps, denom)?)
        }
    }
}

fn query_asset_by_denom(deps: Deps, denom: String) -> StdResult<Option<AssetEntry>> {
    ASSETS_BY_DENOM.may_load(deps.storage, &denom)
}

fn query_asset_by_ticker(deps: Deps, ticker: String) -> StdResult<Option<AssetEntry>> {
    let ticker_norm = normalize_ticker(&ticker);
    if let Some(denom) = DENOM_BY_TICKER.may_load(deps.storage, &ticker_norm)? {
        ASSETS_BY_DENOM.may_load(deps.storage, &denom)
    } else {
        Ok(None)
    }
}

fn query_list_assets(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    include_blocked: Option<bool>,
    only_verified: Option<bool>,
) -> StdResult<Vec<AssetEntry>> {
    let limit = limit.unwrap_or(100).min(500) as usize;
    let include_blocked = include_blocked.unwrap_or(false);
    let only_verified = only_verified.unwrap_or(false);

    let start = start_after.as_deref().map(Bound::exclusive);

    let mut out: Vec<AssetEntry> = Vec::new();

    for item in ASSETS_BY_DENOM
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

fn query_structured_description(deps: Deps, denom: String, key: String) -> StdResult<Option<String>> {
    if let Some(entry) = ASSETS_BY_DENOM.may_load(deps.storage, &denom)? {
        Ok(entry.structured_descriptions.get(&key).cloned())
    } else {
        Ok(None)
    }
}

fn query_all_structured_descriptions(deps: Deps, denom: String) -> StdResult<HashMap<String, String>> {
    if let Some(entry) = ASSETS_BY_DENOM.may_load(deps.storage, &denom)? {
        Ok(entry.structured_descriptions)
    } else {
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn add_asset_enforces_uniqueness() {
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
            ExecuteMsg::AddAsset {
                denom: "ibc/ABC".to_string(),
                name: "Ethereum".to_string(),
                ticker: "eth".to_string(),
                image_url: "https://example.com/eth.svg".to_string(),
                description: "Ethereum".to_string(),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: None,
            },
        )
        .unwrap();

        // Same denom should fail
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddAsset {
                denom: "ibc/ABC".to_string(),
                name: "X".to_string(),
                ticker: "ETH2".to_string(),
                image_url: "https://example.com/x.svg".to_string(),
                description: "X".to_string(),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::DenomAlreadyExists { .. }));

        // Same ticker (normalized) should fail
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::AddAsset {
                denom: "ibc/DEF".to_string(),
                name: "X".to_string(),
                ticker: "ETH".to_string(),
                image_url: "https://example.com/x.svg".to_string(),
                description: "X".to_string(),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::TickerAlreadyExists { .. }));
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
            ExecuteMsg::AddAsset {
                denom: "uusdc".to_string(),
                name: "USD Coin".to_string(),
                ticker: "USDC".to_string(),
                image_url: "https://example.com/usdc.svg".to_string(),
                description: "USD Coin".to_string(),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: Some(6),
            },
        )
        .unwrap();

        // random user cannot update
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("bob", &[]),
            ExecuteMsg::UpdateAsset {
                denom: "uusdc".to_string(),
                name: None,
                image_url: Some("https://example.com/new.svg".to_string()),
                description: None,
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized { .. }));

        // creator can update
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::UpdateAsset {
                denom: "uusdc".to_string(),
                name: Some("USD Coin Display".to_string()),
                image_url: Some("https://example.com/new.svg".to_string()),
                description: Some("USD Coin Updated".to_string()),
                website: Some("https://circle.com".to_string()),
                x: Some("https://x.com/circle".to_string()),
                discord: None,
                telegram: None,
                decimals: Some(6),
            },
        )
        .unwrap();

        // admin can update
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::UpdateAsset {
                denom: "uusdc".to_string(),
                name: Some("Admin Name".to_string()),
                image_url: None,
                description: Some("Admin override".to_string()),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: Some(6),
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
            ExecuteMsg::AddAsset {
                denom: "uatom".to_string(),
                name: "Cosmos Hub".to_string(),
                ticker: "ATOM".to_string(),
                image_url: "https://example.com/atom.svg".to_string(),
                description: "Cosmos Hub".to_string(),
                website: None,
                x: None,
                discord: None,
                telegram: None,
                decimals: Some(6),
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::SetVerified {
                denom: "uatom".to_string(),
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
                denom: "uatom".to_string(),
                verified: true,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            ExecuteMsg::SetBlocked {
                denom: "uatom".to_string(),
                blocked: true,
            },
        )
        .unwrap();

        let asset = query_asset_by_denom(deps.as_ref(), "uatom".to_string())
            .unwrap()
            .unwrap();
        assert!(asset.verified);
        assert!(asset.blocked);
    }
}
