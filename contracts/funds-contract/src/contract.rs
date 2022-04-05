use std::ops::Div;
use std::str::FromStr;

use cosmwasm_std::{Addr, BalanceResponse, BankMsg, BankQuery, Binary, Coin, CosmosMsg, Decimal,
                   Deps, DepsMut, Env, MessageInfo, Order, QuerierWrapper, QueryRequest, Reply,
                   Response, StdResult, SubMsg, to_binary, Uint128, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{BUCKET, KEY_COUNTER, PoolTeamDetails, State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bucket-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        operator: deps.api.addr_validate(&*msg.operator)?.to_string(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    KEY_COUNTER.save(deps.storage, &0)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.operator {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::Sweep {
            funds
        } => sweep(deps, info, funds),
    }
}


pub fn sweep(
    deps: DepsMut,
    info: MessageInfo,
    funds_to_send: Vec<Coin>,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.operator {
        return Err(ContractError::Unauthorized {})
    }
    let to_address = state.operator.to_string().clone();
    let r = CosmosMsg::Bank(BankMsg::Send {
        to_address,
        amount: funds_to_send,
    });
    Ok(Response::new()
        .add_message(r)
        .add_attribute("action", "sweep"))
}
