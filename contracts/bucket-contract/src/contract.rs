use std::ops::Div;
use std::str::FromStr;

use astroport::asset::Asset;
use astroport::pair::QueryMsg::Simulation;
use astroport::pair::SimulationResponse;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Addr, BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Decimal,
                   Deps, DepsMut, Env, MessageInfo, Order, QuerierWrapper, QueryRequest,
                   Reply, Response, StdResult, SubMsg, to_binary, Uint128, WasmMsg};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;
use terra_cosmwasm::TerraQuerier;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
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
        ExecuteMsg::AddToBucket {
            data
        } => add_to_bucket(deps, _env, data),
        ExecuteMsg::RemoveData {
            data
        } => remove_from_bucket(deps, data),
    }
}

// Bucket Execute ---------------------------
fn add_to_bucket(
    deps: DepsMut,
    env: Env,
    data: PoolTeamDetails,
) -> Result<Response, ContractError> {
    let mut counter = KEY_COUNTER.load(deps.storage)?;
    counter += 1;
    KEY_COUNTER.save(deps.storage, &counter)?;
    let new_key = format!("{}_{}", counter, env.block.height);
    BUCKET.save(deps.storage, new_key.clone(), &data)?;
    return Ok(Response::new().add_attribute("method", "add_to_bucket").add_attribute("post_key", new_key));
}

fn remove_from_bucket(
    deps: DepsMut,
    data: PoolTeamDetails,
) -> Result<Response, ContractError> {
    for item in BUCKET.keys(deps.storage, None, None, Order::Ascending).into_iter() {
        let current_item = BUCKET.load(deps.storage, String::from_utf8(item).unwrap())?;
        if current_item == data {
            return Ok(Response::new().add_attribute("method", "remove_from_bucket"));
        }
    }
    return Err(ContractError::DeleteFailed { item: data });
}

// -------------------------------------------
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVector {} => to_binary(&get_vector(deps)?),
        QueryMsg::CheckIfExist { data } => to_binary(&check_in_bucket(deps, data)?),
    }
}
// Bucket Query ------------------------------

fn get_vector(deps: Deps) -> StdResult<Vec<PoolTeamDetails>> {
    let mut data_to_return: Vec<PoolTeamDetails> = Vec::new();
    for item in BUCKET.keys(deps.storage, None, None, Order::Ascending).into_iter() {
        let current_item = BUCKET.load(deps.storage, String::from_utf8(item)?)?;
        data_to_return.push(current_item);
    }
    return Ok(data_to_return);
}

fn check_in_bucket(
    deps: Deps,
    data: PoolTeamDetails,
) -> StdResult<bool> {
    for item in BUCKET.keys(deps.storage, None, None, Order::Ascending).into_iter() {
        let current_item = BUCKET.load(deps.storage, String::from_utf8(item).unwrap()).unwrap();
        if current_item == data {
            return Ok(true);
        }
    }
    return Ok(false);
}

// --------------------------------------------
