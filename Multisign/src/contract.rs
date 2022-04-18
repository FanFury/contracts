use cosmwasm_std::{
    entry_point, to_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,CosmosMsg,WasmMsg, Timestamp, Empty
};

use cw2::set_contract_version;
use cw3_flex_multisig::msg::ExecuteMsg as Cw3ExecuteMsg;
use cw0::Expiration;
use cw20::Cw20ExecuteMsg;

use crate::error::{ContractError};
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config, config_read,save_cw3_address,read_cw3_address, State,read_cw20_address,save_cw20_address};
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
    let mut state = State{
        owner:info.sender.to_string()
    };
    config(deps.storage).save(&state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
       ExecuteMsg::Transfer { address, amount,time,title,description} => execute_transfer(deps,env,info,address,amount,time,title,description),
       ExecuteMsg::SetCw3Address { address } => execute_set_cw3address(deps,env,info,address),
       ExecuteMsg::SetCw20Address { address } => execute_set_cw20address(deps,env,info,address),
    }
}

pub fn execute_transfer(
    deps:DepsMut,
    env:Env,
    info:MessageInfo,
    address:String,
    amount:Uint128,
    time:u64,
    title:String,
    description:String
)->Result<Response, ContractError> {

    let cw3_address = read_cw3_address(deps.storage).load()?;
    let cw20_address = read_cw20_address(deps.storage).load()?;

    let mut propose_msg:Vec<CosmosMsg<Empty>> = vec![];
    
    propose_msg.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw20_address.clone(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient:address,
                amount:amount
            })?,
        })
    );

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw3_address,
            funds: vec![],
            msg: to_binary(&Cw3ExecuteMsg::Propose {
                title:"mint".to_string(),
                description:"mint".to_string(),
                msgs:vec![],
                latest:Some(Expiration::AtTime(Timestamp::from_seconds(time)))
            })?,
        })
    ))
   
    // Ok(Response::new()
    //     .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
    //         contract_addr: prism_address,
    //         funds: vec![deposit_coin],
    //         msg: to_binary(&PrismExecuteMsg::Deposit {
    //         })?,
    //     })
    // ))
}




fn execute_set_cw3address(
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
    save_cw3_address(deps.storage).save(&address)?;
    Ok(Response::default())
}

fn execute_set_cw20address(
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
    save_cw20_address(deps.storage).save(&address)?;
    Ok(Response::default())
}






#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::GetCw3Address {} => to_binary(&query_cw3_address(deps)?),
        QueryMsg::GetCw20Address {} => to_binary(&query_cw20_address(deps)?)
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  config_read(deps.storage).load()?;
    Ok(state)
}

pub fn query_cw3_address(deps:Deps)-> StdResult<String>{
    let cw3_address =  read_cw3_address(deps.storage).load()?;
    Ok(cw3_address)
}

pub fn query_cw20_address(deps:Deps)-> StdResult<String>{
    let cw20_address =  read_cw20_address(deps.storage).load()?;
    Ok(cw20_address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, CosmosMsg, Timestamp,Coin};

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {};
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::SetCw3Address {address: "cw3".to_string() };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let address =  query_cw3_address(deps.as_ref()).unwrap();
        assert_eq!(address,"cw3");

        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::SetCw20Address {address: "cw20".to_string() };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();

        let address =  query_cw20_address(deps.as_ref()).unwrap();
        assert_eq!(address,"cw20");
    
 
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.owner,"creator");

        let info = mock_info("sender",&[]);
        
        let message = ExecuteMsg::Transfer {address:"wallet".to_string(),amount:Uint128::new(10),time:60000, title:"mint".to_string(),description:"mint".to_string() };
         let res= execute(deps.as_mut(), mock_env(), info, message).unwrap();
        assert_eq!(res.messages.len(),1);

          let mut propose_msg:Vec<CosmosMsg<Empty>> = vec![];
    
         propose_msg.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "cw20".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient:"wallet".to_string(),
                amount:Uint128::new(10),
            }).unwrap(),
            })
        );
        
        assert_eq!(res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "cw3".to_string(),
                funds: vec![],
                  msg: to_binary(&Cw3ExecuteMsg::Propose {
                title:"mint".to_string(),
                description:"mint".to_string(),
                msgs:vec![],
                latest:Some(Expiration::AtTime(Timestamp::from_seconds(60000)))
            }).unwrap(),
        }))
    }
}
