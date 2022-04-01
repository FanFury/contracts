use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,CosmosMsg,WasmMsg
};

use cw2::set_contract_version;

use crate::error::{ContractError};
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config, config_read,save_prism_address,read_prism_address, State};
use crate::prism::{ExecuteMsg as PrismExecuteMsg};

const CONTRACT_NAME: &str = "my-wallet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    deps.api.addr_validate(&msg.owner)?;
    let state = State {
       owner:msg.owner,
       denom: msg.denom
    };
    config(deps.storage).save(&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    match msg {
       ExecuteMsg::Deposit {} => execute_deposit(deps,env,info),
       ExecuteMsg::SetOwner {address} => execute_set_owner(deps,env,info,address),
       ExecuteMsg::ChangeDenom { denom } => execute_change_denom(deps,env,info,denom),
       ExecuteMsg::SetPrismAddress { address } => execute_set_prism(deps,env,info,address),
    }
}

pub fn execute_deposit(
    deps:DepsMut,
    env:Env,
    info:MessageInfo
)->Result<Response, ContractError> {
    let state = config_read(deps.storage).load()?;
    let prism_address = read_prism_address(deps.storage).load()?;
    let deposit_amount  = info
        .funds
        .iter()
        .find(|c| c.denom == state.denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    let deposit_coin = Coin{
        denom:state.denom,
        amount:deposit_amount};
 
    if deposit_amount == Uint128 :: new(0) {
        return Err(ContractError::NotEnough { });
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: prism_address,
            funds: vec![deposit_coin],
            msg: to_binary(&PrismExecuteMsg::Deposit {
            })?,
        })
    ))
}

fn execute_set_owner(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;

    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    deps.api.addr_validate(&address)?;
    state.owner = address;
    config(deps.storage).save(&state)?;
    Ok(Response::default())
}

fn execute_set_prism(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    deps.api.addr_validate(&address)?;
    save_prism_address(deps.storage).save(&address)?;
    Ok(Response::default())
}

fn execute_change_denom(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    denom: String,
) -> Result<Response, ContractError> {
    let mut state = config_read(deps.storage).load()?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    state.denom = denom;
    config(deps.storage).save(&state)?;
    Ok(Response::default())
}



#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::GetPrismAddress {} => to_binary(&query_prism_address(deps)?)
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  config_read(deps.storage).load()?;
    Ok(state)
}

pub fn query_prism_address(deps:Deps)-> StdResult<String>{
    let prism_address =  read_prism_address(deps.storage).load()?;
    Ok(prism_address)
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, CosmosMsg, Timestamp,Coin};

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {owner:String::from("creator"),denom:String::from("UST")};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::SetPrismAddress {address: "prism".to_string() };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let address =  query_prism_address(deps.as_ref()).unwrap();
        assert_eq!(address,"prism");

        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.denom,"UST".to_string());

        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::SetOwner  {address: "creator1".to_string() };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();
        
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.owner,"creator1");

        let info = mock_info("sender",&[Coin{
            denom:"aaa".to_string(),
            amount:Uint128::new(10)
        },Coin{
            denom:"UST".to_string(),
            amount:Uint128::new(20)
        }]);
        
        let message = ExecuteMsg::Deposit { };
         let res= execute(deps.as_mut(), mock_env(), info, message).unwrap();
        assert_eq!(res.messages.len(),1);
        assert_eq!(res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "prism".to_string(),
                funds: vec![Coin{
                    denom:"UST".to_string(),
                    amount:Uint128::new(20)
                }],
                msg: to_binary(&PrismExecuteMsg::Deposit {
            }).unwrap(),
        }))
    }
}
