#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, InvoiceResponse, InvoicesResponse, QueryMsg};
use crate::state::{Config, Invoice, InvoiceStatus, CONFIG, INVOICES, INVOICE_COUNTER, RECEIVER_INVOICES, SENDER_INVOICES};

const CONTRACT_NAME: &str = "crates.io:payment-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    
    let config = Config {
        user_registry: msg.user_registry,
    };
    CONFIG.save(deps.storage, &config)?;
    INVOICE_COUNTER.save(deps.storage, &0u64)?;
    
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
        ExecuteMsg::CreateInvoice {
            invoice_id,
            amount,
            denom,
            reference,
            expires_in,
        } => execute_create_invoice(deps, env, info, invoice_id, amount, denom, reference, expires_in),
        ExecuteMsg::FundInvoice { invoice_id } => execute_fund_invoice(deps, env, info, invoice_id),
        ExecuteMsg::ReleaseFunds { invoice_id } => execute_release_funds(deps, env, info, invoice_id),
        ExecuteMsg::RefundFunds { invoice_id } => execute_refund_funds(deps, env, info, invoice_id),
        ExecuteMsg::CancelInvoice { invoice_id } => execute_cancel_invoice(deps, env, info, invoice_id),
        ExecuteMsg::UpdateConfig { user_registry } => execute_update_config(deps, info, user_registry),
    }
}

fn execute_create_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    invoice_id: String,
    amount: u128,
    denom: String,
    reference: String,
    expires_in: Option<u64>,
) -> Result<Response, ContractError> {
    // Check if invoice ID already exists
    if INVOICES.has(deps.storage, &invoice_id) {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            format!("Invoice ID {} already exists", invoice_id),
        )));
    }
    
    let receiver = info.sender.to_string();
    let now = env.block.time.seconds();
    let expires_at = expires_in.map(|e| now + e).unwrap_or(0);
    
    let invoice = Invoice {
        invoice_id: invoice_id.clone(),
        receiver: receiver.clone(),
        sender: None,
        amount,
        denom: denom.clone(),
        reference: reference.clone(),
        status: InvoiceStatus::Pending,
        created_at: now,
        funded_at: None,
        completed_at: None,
        expires_at,
    };
    
    // Save invoice
    INVOICES.save(deps.storage, &invoice_id, &invoice)?;
    
    // Add to receiver's invoice list
    let mut receiver_invoices = RECEIVER_INVOICES
        .may_load(deps.storage, &receiver)?
        .unwrap_or_default();
    receiver_invoices.push(invoice_id.clone());
    RECEIVER_INVOICES.save(deps.storage, &receiver, &receiver_invoices)?;
    
    Ok(Response::new()
        .add_attribute("method", "create_invoice")
        .add_attribute("invoice_id", invoice_id)
        .add_attribute("receiver", receiver)
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom)
        .add_attribute("reference", reference))
}

fn execute_fund_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    invoice_id: String,
) -> Result<Response, ContractError> {
    // Load invoice
    let mut invoice = INVOICES
        .may_load(deps.storage, &invoice_id)?
        .ok_or(ContractError::InvoiceNotFound { invoice_id: invoice_id.clone() })?;
    
    // Check status
    match invoice.status {
        InvoiceStatus::Pending => {}
        InvoiceStatus::Funded => return Err(ContractError::AlreadyFunded {}),
        _ => return Err(ContractError::AlreadyCompleted {}),
    }
    
    // Check expiry
    if invoice.expires_at > 0 && env.block.time.seconds() > invoice.expires_at {
        return Err(ContractError::Expired {});
    }
    
    // Verify payment
    let expected_coin = Coin {
        denom: invoice.denom.clone(),
        amount: invoice.amount.into(),
    };
    
    let sent = info.funds.iter().find(|c| c.denom == invoice.denom);
    match sent {
        None => {
            return Err(ContractError::IncorrectDenom {
                expected: invoice.denom.clone(),
                received: info.funds.first().map(|c| c.denom.clone()).unwrap_or_default(),
            });
        }
        Some(coin) => {
            if coin.amount.u128() != invoice.amount {
                return Err(ContractError::IncorrectAmount {
                    expected: invoice.amount.to_string(),
                    received: coin.amount.to_string(),
                });
            }
        }
    }
    
    let sender = info.sender.to_string();
    
    // Update invoice
    invoice.sender = Some(sender.clone());
    invoice.status = InvoiceStatus::Funded;
    invoice.funded_at = Some(env.block.time.seconds());
    INVOICES.save(deps.storage, &invoice_id, &invoice)?;
    
    // Add to sender's invoice list
    let mut sender_invoices = SENDER_INVOICES
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();
    sender_invoices.push(invoice_id.clone());
    SENDER_INVOICES.save(deps.storage, &sender, &sender_invoices)?;
    
    Ok(Response::new()
        .add_attribute("method", "fund_invoice")
        .add_attribute("invoice_id", invoice_id)
        .add_attribute("sender", sender)
        .add_attribute("amount", invoice.amount.to_string())
        .add_attribute("denom", invoice.denom))
}

fn execute_release_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    invoice_id: String,
) -> Result<Response, ContractError> {
    let mut invoice = INVOICES
        .may_load(deps.storage, &invoice_id)?
        .ok_or(ContractError::InvoiceNotFound { invoice_id: invoice_id.clone() })?;
    
    // Only receiver can release funds
    if info.sender.to_string() != invoice.receiver {
        return Err(ContractError::OnlyReceiver {});
    }
    
    // Check status
    if invoice.status != InvoiceStatus::Funded {
        return Err(ContractError::NotFunded {});
    }
    
    // Update status
    invoice.status = InvoiceStatus::Completed;
    invoice.completed_at = Some(env.block.time.seconds());
    INVOICES.save(deps.storage, &invoice_id, &invoice)?;
    
    // Send funds to receiver
    let send_msg = BankMsg::Send {
        to_address: invoice.receiver.clone(),
        amount: vec![Coin {
            denom: invoice.denom.clone(),
            amount: invoice.amount.into(),
        }],
    };
    
    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("method", "release_funds")
        .add_attribute("invoice_id", invoice_id)
        .add_attribute("receiver", invoice.receiver)
        .add_attribute("amount", invoice.amount.to_string()))
}

fn execute_refund_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    invoice_id: String,
) -> Result<Response, ContractError> {
    let mut invoice = INVOICES
        .may_load(deps.storage, &invoice_id)?
        .ok_or(ContractError::InvoiceNotFound { invoice_id: invoice_id.clone() })?;
    
    // Only receiver can refund
    if info.sender.to_string() != invoice.receiver {
        return Err(ContractError::OnlyReceiver {});
    }
    
    // Check status
    if invoice.status != InvoiceStatus::Funded {
        return Err(ContractError::NotFunded {});
    }
    
    let sender = invoice.sender.clone().ok_or(ContractError::NotFunded {})?;
    
    // Update status
    invoice.status = InvoiceStatus::Refunded;
    invoice.completed_at = Some(env.block.time.seconds());
    INVOICES.save(deps.storage, &invoice_id, &invoice)?;
    
    // Send funds back to sender
    let send_msg = BankMsg::Send {
        to_address: sender.clone(),
        amount: vec![Coin {
            denom: invoice.denom.clone(),
            amount: invoice.amount.into(),
        }],
    };
    
    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("method", "refund_funds")
        .add_attribute("invoice_id", invoice_id)
        .add_attribute("sender", sender)
        .add_attribute("amount", invoice.amount.to_string()))
}

fn execute_cancel_invoice(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    invoice_id: String,
) -> Result<Response, ContractError> {
    let mut invoice = INVOICES
        .may_load(deps.storage, &invoice_id)?
        .ok_or(ContractError::InvoiceNotFound { invoice_id: invoice_id.clone() })?;
    
    // Only receiver can cancel
    if info.sender.to_string() != invoice.receiver {
        return Err(ContractError::OnlyReceiver {});
    }
    
    // Can only cancel pending invoices
    if invoice.status != InvoiceStatus::Pending {
        return Err(ContractError::AlreadyFunded {});
    }
    
    // Update status
    invoice.status = InvoiceStatus::Cancelled;
    invoice.completed_at = Some(env.block.time.seconds());
    INVOICES.save(deps.storage, &invoice_id, &invoice)?;
    
    Ok(Response::new()
        .add_attribute("method", "cancel_invoice")
        .add_attribute("invoice_id", invoice_id))
}

fn execute_update_config(
    deps: DepsMut,
    _info: MessageInfo,
    user_registry: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    config.user_registry = user_registry;
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new().add_attribute("method", "update_config"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInvoice { invoice_id } => to_json_binary(&query_invoice(deps, invoice_id)?),
        QueryMsg::GetReceiverInvoices { receiver, status } => {
            to_json_binary(&query_receiver_invoices(deps, receiver, status)?)
        }
        QueryMsg::GetSenderInvoices { sender, status } => {
            to_json_binary(&query_sender_invoices(deps, sender, status)?)
        }
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
    }
}

fn query_invoice(deps: Deps, invoice_id: String) -> StdResult<InvoiceResponse> {
    let invoice = INVOICES.may_load(deps.storage, &invoice_id)?;
    Ok(InvoiceResponse { invoice })
}

fn query_receiver_invoices(
    deps: Deps,
    receiver: String,
    status_filter: Option<InvoiceStatus>,
) -> StdResult<InvoicesResponse> {
    let invoice_ids = RECEIVER_INVOICES
        .may_load(deps.storage, &receiver)?
        .unwrap_or_default();
    
    let mut invoices = Vec::new();
    for id in invoice_ids {
        if let Some(invoice) = INVOICES.may_load(deps.storage, &id)? {
            if let Some(ref status) = status_filter {
                if &invoice.status == status {
                    invoices.push(invoice);
                }
            } else {
                invoices.push(invoice);
            }
        }
    }
    
    Ok(InvoicesResponse { invoices })
}

fn query_sender_invoices(
    deps: Deps,
    sender: String,
    status_filter: Option<InvoiceStatus>,
) -> StdResult<InvoicesResponse> {
    let invoice_ids = SENDER_INVOICES
        .may_load(deps.storage, &sender)?
        .unwrap_or_default();
    
    let mut invoices = Vec::new();
    for id in invoice_ids {
        if let Some(invoice) = INVOICES.may_load(deps.storage, &id)? {
            if let Some(ref status) = status_filter {
                if &invoice.status == status {
                    invoices.push(invoice);
                }
            } else {
                invoices.push(invoice);
            }
        }
    }
    
    Ok(InvoicesResponse { invoices })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        user_registry: config.user_registry,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::coins;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { user_registry: None };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn create_and_fund_invoice() {
        let mut deps = mock_dependencies();
        
        // Instantiate
        let msg = InstantiateMsg { user_registry: None };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Create invoice
        let info = mock_info("receiver", &[]);
        let msg = ExecuteMsg::CreateInvoice {
            invoice_id: "inv_001".to_string(),
            amount: 1000000,
            denom: "uusdc".to_string(),
            reference: "Test payment".to_string(),
            expires_in: None,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Query invoice
        let res = query_invoice(deps.as_ref(), "inv_001".to_string()).unwrap();
        assert!(res.invoice.is_some());
        let invoice = res.invoice.unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Pending);
        assert_eq!(invoice.amount, 1000000);
        
        // Fund invoice
        let info = mock_info("sender", &coins(1000000, "uusdc"));
        let msg = ExecuteMsg::FundInvoice {
            invoice_id: "inv_001".to_string(),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Check funded status
        let res = query_invoice(deps.as_ref(), "inv_001".to_string()).unwrap();
        let invoice = res.invoice.unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Funded);
        assert_eq!(invoice.sender, Some("sender".to_string()));
    }

    #[test]
    fn release_funds() {
        let mut deps = mock_dependencies();
        
        // Setup
        let msg = InstantiateMsg { user_registry: None };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
        // Create and fund
        let info = mock_info("receiver", &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::CreateInvoice {
            invoice_id: "inv_001".to_string(),
            amount: 1000000,
            denom: "uusdc".to_string(),
            reference: "Test".to_string(),
            expires_in: None,
        }).unwrap();
        
        let info = mock_info("sender", &coins(1000000, "uusdc"));
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::FundInvoice {
            invoice_id: "inv_001".to_string(),
        }).unwrap();
        
        // Release funds
        let info = mock_info("receiver", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::ReleaseFunds {
            invoice_id: "inv_001".to_string(),
        }).unwrap();
        
        // Check bank message was created
        assert_eq!(res.messages.len(), 1);
        
        // Check status
        let res = query_invoice(deps.as_ref(), "inv_001".to_string()).unwrap();
        let invoice = res.invoice.unwrap();
        assert_eq!(invoice.status, InvoiceStatus::Completed);
    }
}
