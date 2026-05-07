#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultResponse, VaultsResponse};
use crate::state::{
    Vault, VaultStatus, COMPLETED_INDEX, COUNTERPARTY_INDEX, CREATOR_INDEX, OPEN_INDEX, VAULTS,
};

const CONTRACT_NAME: &str = "crates.io:otc-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------- bounds ----------
const MAX_VAULT_ID_LEN: usize = 64;
const MIN_VAULT_ID_LEN: usize = 1;
const MAX_DESCRIPTION_LEN: usize = 256;
const MAX_DENOM_LEN: usize = 128;
const DEFAULT_QUERY_LIMIT: u32 = 30;
const MAX_QUERY_LIMIT: u32 = 100;

// ---------- entry points ----------

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
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateVault {
            vault_id,
            ask_amount,
            ask_denom,
            description,
            expires_in,
        } => execute_create_vault(
            deps,
            env,
            info,
            vault_id,
            ask_amount,
            ask_denom,
            description,
            expires_in,
        ),
        ExecuteMsg::FundVault { vault_id } => execute_fund_vault(deps, env, info, vault_id),
        ExecuteMsg::CancelVault { vault_id } => execute_cancel_vault(deps, env, info, vault_id),
    }
}

// ---------- helpers ----------

fn check_len(field: &str, value: &str, min: usize, max: usize) -> Result<(), ContractError> {
    let len = value.len();
    if len < min || len > max {
        return Err(ContractError::InvalidLength {
            field: field.to_string(),
            actual: len,
            min,
            max,
        });
    }
    Ok(())
}

fn clamp_limit(limit: Option<u32>) -> usize {
    limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .clamp(1, MAX_QUERY_LIMIT) as usize
}

// ---------- execute ----------

#[allow(clippy::too_many_arguments)]
fn execute_create_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: String,
    ask_amount: Uint128,
    ask_denom: String,
    description: String,
    expires_in: Option<u64>,
) -> Result<Response, ContractError> {
    // Validate field lengths.
    check_len("vault_id", &vault_id, MIN_VAULT_ID_LEN, MAX_VAULT_ID_LEN)?;
    check_len("description", &description, 0, MAX_DESCRIPTION_LEN)?;
    check_len("ask_denom", &ask_denom, 1, MAX_DENOM_LEN)?;

    // Unique vault id.
    if VAULTS.has(deps.storage, &vault_id) {
        return Err(ContractError::VaultAlreadyExists { vault_id });
    }

    // Must send exactly one coin (the offer).
    if info.funds.len() != 1 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Must send exactly one coin as the offer",
        )));
    }
    let offer_coin = &info.funds[0];
    let offer_amount = offer_coin.amount;
    let offer_denom = offer_coin.denom.clone();

    check_len("offer_denom", &offer_denom, 1, MAX_DENOM_LEN)?;

    // Cannot swap same denom.
    if offer_denom == ask_denom {
        return Err(ContractError::SameDenom { denom: offer_denom });
    }

    // Validate non-zero amounts.
    if offer_amount.is_zero() || ask_amount.is_zero() {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Amounts must be greater than zero",
        )));
    }

    // Creator is the message sender (already validated by the framework).
    let creator = info.sender.to_string();
    let now = env.block.time.seconds();
    let expires_at = expires_in
        .filter(|e| *e > 0)
        .map(|e| now.saturating_add(e))
        .unwrap_or(0);

    let vault = Vault {
        vault_id: vault_id.clone(),
        creator: creator.clone(),
        counterparty: None,
        offer_amount,
        offer_denom: offer_denom.clone(),
        ask_amount,
        ask_denom: ask_denom.clone(),
        description: description.clone(),
        status: VaultStatus::Open,
        created_at: now,
        creator_funded_at: Some(now),
        completed_at: None,
        expires_at,
    };

    VAULTS.save(deps.storage, &vault_id, &vault)?;
    OPEN_INDEX.save(deps.storage, &vault_id, &())?;
    CREATOR_INDEX.save(deps.storage, (&creator, &vault_id), &())?;

    Ok(Response::new()
        .add_attribute("method", "create_vault")
        .add_attribute("vault_id", vault_id)
        .add_attribute("creator", creator)
        .add_attribute("offer_amount", offer_amount.to_string())
        .add_attribute("offer_denom", offer_denom)
        .add_attribute("ask_amount", ask_amount.to_string())
        .add_attribute("ask_denom", ask_denom)
        .add_attribute("description", description))
}

fn execute_fund_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: String,
) -> Result<Response, ContractError> {
    let mut vault = VAULTS.may_load(deps.storage, &vault_id)?.ok_or_else(|| {
        ContractError::VaultNotFound {
            vault_id: vault_id.clone(),
        }
    })?;

    // Must be Open.
    if vault.status != VaultStatus::Open {
        return Err(ContractError::NotOpen {});
    }

    // Expiry check.
    if vault.expires_at > 0 && env.block.time.seconds() > vault.expires_at {
        return Err(ContractError::Expired {});
    }

    // Creator cannot fund their own vault as counterparty.
    let counterparty = info.sender.to_string();
    if counterparty == vault.creator {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Creator cannot be counterparty",
        )));
    }

    // Strict funds validation: exactly one coin, matching denom and amount.
    if info.funds.len() != 1 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "Must send exactly one coin matching (ask_denom, ask_amount)",
        )));
    }
    let coin = &info.funds[0];
    if coin.denom != vault.ask_denom {
        return Err(ContractError::IncorrectDenom {
            expected: vault.ask_denom.clone(),
            received: coin.denom.clone(),
        });
    }
    if coin.amount != vault.ask_amount {
        return Err(ContractError::IncorrectAmount {
            expected: vault.ask_amount.to_string(),
            received: coin.amount.to_string(),
        });
    }

    // Update vault state.
    let completed_at = env.block.time.seconds();
    vault.counterparty = Some(counterparty.clone());
    vault.status = VaultStatus::Completed;
    vault.completed_at = Some(completed_at);
    VAULTS.save(deps.storage, &vault_id, &vault)?;

    // Maintain indexes.
    OPEN_INDEX.remove(deps.storage, &vault_id);
    COMPLETED_INDEX.save(deps.storage, (completed_at, &vault_id), &())?;
    COUNTERPARTY_INDEX.save(deps.storage, (&counterparty, &vault_id), &())?;

    // Atomic swap: two BankMsg::Send.
    let send_to_counterparty = BankMsg::Send {
        to_address: counterparty.clone(),
        amount: vec![Coin {
            denom: vault.offer_denom.clone(),
            amount: vault.offer_amount,
        }],
    };
    let send_to_creator = BankMsg::Send {
        to_address: vault.creator.clone(),
        amount: vec![Coin {
            denom: vault.ask_denom.clone(),
            amount: vault.ask_amount,
        }],
    };

    Ok(Response::new()
        .add_message(send_to_counterparty)
        .add_message(send_to_creator)
        .add_attribute("method", "fund_vault")
        .add_attribute("vault_id", vault_id)
        .add_attribute("counterparty", counterparty)
        .add_attribute(
            "creator_receives",
            format!("{} {}", vault.ask_amount, vault.ask_denom),
        )
        .add_attribute(
            "counterparty_receives",
            format!("{} {}", vault.offer_amount, vault.offer_denom),
        ))
}

fn execute_cancel_vault(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vault_id: String,
) -> Result<Response, ContractError> {
    let mut vault = VAULTS.may_load(deps.storage, &vault_id)?.ok_or_else(|| {
        ContractError::VaultNotFound {
            vault_id: vault_id.clone(),
        }
    })?;

    // Only creator can cancel.
    if info.sender.to_string() != vault.creator {
        return Err(ContractError::OnlyCreator {});
    }

    // Only while Open.
    match vault.status {
        VaultStatus::Open => {}
        VaultStatus::Completed => return Err(ContractError::AlreadyCompleted {}),
        VaultStatus::Cancelled => return Err(ContractError::AlreadyCompleted {}),
    }

    vault.status = VaultStatus::Cancelled;
    vault.completed_at = Some(env.block.time.seconds());
    VAULTS.save(deps.storage, &vault_id, &vault)?;

    // Maintain indexes: drop from OPEN, keep in CREATOR_INDEX for history.
    OPEN_INDEX.remove(deps.storage, &vault_id);

    // Return creator's offered funds.
    let refund_msg = BankMsg::Send {
        to_address: vault.creator.clone(),
        amount: vec![Coin {
            denom: vault.offer_denom.clone(),
            amount: vault.offer_amount,
        }],
    };

    Ok(Response::new()
        .add_message(refund_msg)
        .add_attribute("method", "cancel_vault")
        .add_attribute("vault_id", vault_id)
        .add_attribute("refunded_to", vault.creator)
        .add_attribute(
            "refunded_amount",
            format!("{} {}", vault.offer_amount, vault.offer_denom),
        ))
}

// ---------- query ----------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVault { vault_id } => to_json_binary(&query_vault(deps, vault_id)?),
        QueryMsg::GetOpenVaults {
            offer_denom,
            ask_denom,
            start_after,
            limit,
        } => to_json_binary(&query_open_vaults(
            deps,
            env,
            offer_denom,
            ask_denom,
            start_after,
            limit,
        )?),
        QueryMsg::GetCompletedVaults { limit } => {
            to_json_binary(&query_completed_vaults(deps, limit)?)
        }
        QueryMsg::GetCreatorVaults {
            creator,
            status,
            limit,
        } => to_json_binary(&query_creator_vaults(deps, creator, status, limit)?),
        QueryMsg::GetCounterpartyVaults {
            counterparty,
            limit,
        } => to_json_binary(&query_counterparty_vaults(deps, counterparty, limit)?),
    }
}

fn query_vault(deps: Deps, vault_id: String) -> StdResult<VaultResponse> {
    Ok(VaultResponse {
        vault: VAULTS.may_load(deps.storage, &vault_id)?,
    })
}

fn query_open_vaults(
    deps: Deps,
    env: Env,
    offer_denom: Option<String>,
    ask_denom: Option<String>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<VaultsResponse> {
    let limit = clamp_limit(limit);
    let now = env.block.time.seconds();
    let start = start_after.as_deref().map(Bound::exclusive);

    // Walk the OPEN_INDEX; load each vault; filter. Cap iterations at limit * 4
    // so denom filters don't drag us through the whole index.
    let max_scan = limit.saturating_mul(4);
    let mut vaults = Vec::with_capacity(limit);
    for item in OPEN_INDEX
        .range(deps.storage, start, None, Order::Ascending)
        .take(max_scan)
    {
        let (vault_id, _) = item?;
        let vault = match VAULTS.may_load(deps.storage, &vault_id)? {
            Some(v) => v,
            None => continue,
        };

        // Skip expired.
        if vault.expires_at > 0 && now > vault.expires_at {
            continue;
        }
        if let Some(ref od) = offer_denom {
            if &vault.offer_denom != od {
                continue;
            }
        }
        if let Some(ref ad) = ask_denom {
            if &vault.ask_denom != ad {
                continue;
            }
        }

        vaults.push(vault);
        if vaults.len() >= limit {
            break;
        }
    }

    Ok(VaultsResponse { vaults })
}

fn query_completed_vaults(deps: Deps, limit: Option<u32>) -> StdResult<VaultsResponse> {
    let limit = clamp_limit(limit);
    let mut vaults = Vec::with_capacity(limit);
    for item in COMPLETED_INDEX
        .range(deps.storage, None, None, Order::Descending)
        .take(limit)
    {
        let ((_, vault_id), _) = item?;
        if let Some(v) = VAULTS.may_load(deps.storage, &vault_id)? {
            vaults.push(v);
        }
    }
    Ok(VaultsResponse { vaults })
}

fn query_creator_vaults(
    deps: Deps,
    creator: String,
    status_filter: Option<VaultStatus>,
    limit: Option<u32>,
) -> StdResult<VaultsResponse> {
    let limit = clamp_limit(limit);
    let mut vaults = Vec::with_capacity(limit);
    for item in CREATOR_INDEX
        .prefix(&creator)
        .range(deps.storage, None, None, Order::Ascending)
    {
        let (vault_id, _) = item?;
        if let Some(v) = VAULTS.may_load(deps.storage, &vault_id)? {
            if let Some(ref s) = status_filter {
                if &v.status != s {
                    continue;
                }
            }
            vaults.push(v);
            if vaults.len() >= limit {
                break;
            }
        }
    }
    Ok(VaultsResponse { vaults })
}

fn query_counterparty_vaults(
    deps: Deps,
    counterparty: String,
    limit: Option<u32>,
) -> StdResult<VaultsResponse> {
    let limit = clamp_limit(limit);
    let mut vaults = Vec::with_capacity(limit);
    for item in COUNTERPARTY_INDEX
        .prefix(&counterparty)
        .range(deps.storage, None, None, Order::Ascending)
        .take(limit)
    {
        let (vault_id, _) = item?;
        if let Some(v) = VAULTS.may_load(deps.storage, &vault_id)? {
            vaults.push(v);
        }
    }
    Ok(VaultsResponse { vaults })
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn create_and_fund_vault() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        // Create vault: offering 2 ATOM for 4 USDC.
        let info = mock_info("alice", &coins(2_000_000, "uatom"));
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::CreateVault {
                vault_id: "vault_001".to_string(),
                ask_amount: Uint128::new(4_000_000),
                ask_denom: "uusdc".to_string(),
                description: "2 ATOM for 4 USDC".to_string(),
                expires_in: None,
            },
        )
        .unwrap();

        let v = query_vault(deps.as_ref(), "vault_001".to_string())
            .unwrap()
            .vault
            .unwrap();
        assert_eq!(v.status, VaultStatus::Open);
        assert_eq!(v.offer_amount, Uint128::new(2_000_000));

        // Fund as bob.
        let info = mock_info("bob", &coins(4_000_000, "uusdc"));
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::FundVault {
                vault_id: "vault_001".to_string(),
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 2);

        let v = query_vault(deps.as_ref(), "vault_001".to_string())
            .unwrap()
            .vault
            .unwrap();
        assert_eq!(v.status, VaultStatus::Completed);
        assert_eq!(v.counterparty, Some("bob".to_string()));
    }

    #[test]
    fn fund_rejects_extra_coins() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(100, "uatom")),
            ExecuteMsg::CreateVault {
                vault_id: "v".to_string(),
                ask_amount: Uint128::new(200),
                ask_denom: "uusdc".to_string(),
                description: String::new(),
                expires_in: None,
            },
        )
        .unwrap();

        // Attach two coins: correct ask + extra dust. Must be rejected.
        let info = mock_info(
            "bob",
            &[
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(200),
                },
                Coin {
                    denom: "ujunk".to_string(),
                    amount: Uint128::new(1),
                },
            ],
        );
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::FundVault {
                vault_id: "v".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Std(_)));
    }

    #[test]
    fn cancel_vault() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(2_000_000, "uatom")),
            ExecuteMsg::CreateVault {
                vault_id: "v".to_string(),
                ask_amount: Uint128::new(4_000_000),
                ask_denom: "uusdc".to_string(),
                description: "Test".to_string(),
                expires_in: None,
            },
        )
        .unwrap();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &[]),
            ExecuteMsg::CancelVault {
                vault_id: "v".to_string(),
            },
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);

        let v = query_vault(deps.as_ref(), "v".to_string())
            .unwrap()
            .vault
            .unwrap();
        assert_eq!(v.status, VaultStatus::Cancelled);
    }

    #[test]
    fn cannot_swap_same_denom() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(2_000_000, "uatom")),
            ExecuteMsg::CreateVault {
                vault_id: "v".to_string(),
                ask_amount: Uint128::new(4_000_000),
                ask_denom: "uatom".to_string(),
                description: "Invalid".to_string(),
                expires_in: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::SameDenom { .. }));
    }

    #[test]
    fn creator_cannot_be_counterparty() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(2_000_000, "uatom")),
            ExecuteMsg::CreateVault {
                vault_id: "v".to_string(),
                ask_amount: Uint128::new(4_000_000),
                ask_denom: "uusdc".to_string(),
                description: "Test".to_string(),
                expires_in: None,
            },
        )
        .unwrap();

        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(4_000_000, "uusdc")),
            ExecuteMsg::FundVault {
                vault_id: "v".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Std(_)));
    }

    #[test]
    fn rejects_oversized_vault_id() {
        let mut deps = mock_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info("admin", &[]),
            InstantiateMsg {},
        )
        .unwrap();

        let too_long = "x".repeat(MAX_VAULT_ID_LEN + 1);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("alice", &coins(1, "uatom")),
            ExecuteMsg::CreateVault {
                vault_id: too_long,
                ask_amount: Uint128::new(1),
                ask_denom: "uusdc".to_string(),
                description: String::new(),
                expires_in: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidLength { .. }));
    }
}
