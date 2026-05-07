#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, ProfileResponse, QueryMsg};
use crate::state::{UserProfile, PROFILES};

const CONTRACT_NAME: &str = "crates.io:user-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateProfile {
            profile_picture_ipfs,
            name,
            company_name,
            address,
            city,
            phone,
            email,
            website,
            tax_id,
        } => execute_update_profile(
            deps,
            info,
            profile_picture_ipfs,
            name,
            company_name,
            address,
            city,
            phone,
            email,
            website,
            tax_id,
        ),
        ExecuteMsg::DeleteProfile {} => execute_delete_profile(deps, info),
    }
}

fn execute_update_profile(
    deps: DepsMut,
    info: MessageInfo,
    profile_picture_ipfs: Option<String>,
    name: Option<String>,
    company_name: Option<String>,
    address: Option<String>,
    city: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    website: Option<String>,
    tax_id: Option<String>,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    // Load existing profile or create new one
    let mut profile = PROFILES
        .may_load(deps.storage, &sender)?
        .unwrap_or_else(|| UserProfile::new(sender.clone()));
    
    // Update fields if provided
    if profile_picture_ipfs.is_some() {
        profile.profile_picture_ipfs = profile_picture_ipfs;
    }
    if name.is_some() {
        profile.name = name;
    }
    if company_name.is_some() {
        profile.company_name = company_name;
    }
    if address.is_some() {
        profile.address = address;
    }
    if city.is_some() {
        profile.city = city;
    }
    if phone.is_some() {
        profile.phone = phone;
    }
    if email.is_some() {
        profile.email = email;
    }
    if website.is_some() {
        profile.website = website;
    }
    if tax_id.is_some() {
        profile.tax_id = tax_id;
    }
    
    // Save profile
    PROFILES.save(deps.storage, &sender, &profile)?;
    
    Ok(Response::new()
        .add_attribute("method", "update_profile")
        .add_attribute("address", sender))
}

fn execute_delete_profile(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();
    
    // Check if profile exists
    if !PROFILES.has(deps.storage, &sender) {
        return Err(ContractError::ProfileNotFound { address: sender });
    }
    
    // Remove profile
    PROFILES.remove(deps.storage, &sender);
    
    Ok(Response::new()
        .add_attribute("method", "delete_profile")
        .add_attribute("address", sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProfile { address } => to_json_binary(&query_profile(deps, address)?),
        QueryMsg::ProfileExists { address } => to_json_binary(&query_profile_exists(deps, address)?),
    }
}

fn query_profile(deps: Deps, address: String) -> StdResult<ProfileResponse> {
    let profile = PROFILES.may_load(deps.storage, &address)?;
    Ok(ProfileResponse { profile })
}

fn query_profile_exists(deps: Deps, address: String) -> StdResult<bool> {
    Ok(PROFILES.has(deps.storage, &address))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn update_profile() {
        let mut deps = mock_dependencies();
        
        // Instantiate
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Update profile
        let info = mock_info("user1", &[]);
        let msg = ExecuteMsg::UpdateProfile {
            profile_picture_ipfs: None,
            name: Some("John Doe".to_string()),
            company_name: Some("ACME Inc".to_string()),
            address: None,
            city: Some("New York".to_string()),
            phone: None,
            email: Some("john@example.com".to_string()),
            website: None,
            tax_id: None,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Query profile
        let res = query_profile(deps.as_ref(), "user1".to_string()).unwrap();
        assert!(res.profile.is_some());
        let profile = res.profile.unwrap();
        assert_eq!(profile.name, Some("John Doe".to_string()));
        assert_eq!(profile.company_name, Some("ACME Inc".to_string()));
        assert_eq!(profile.city, Some("New York".to_string()));
        assert_eq!(profile.email, Some("john@example.com".to_string()));
    }
}
