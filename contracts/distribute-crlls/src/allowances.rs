use cosmwasm_std::{
    attr, Addr, Binary, BlockInfo, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage, Uint128,
};
use cw20::{AllowanceResponse, Cw20ReceiveMsg, Expiration};

use crate::error::ContractError;
use crate::state::{ALLOWANCES, BALANCES};

pub fn execute_increase_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    if spender_addr == info.sender {
        return Err(ContractError::CannotSetOwnAccount {});
    }

    ALLOWANCES.update(
        deps.storage,
        (&info.sender, &spender_addr),
        |allow| -> StdResult<_> {
            let mut val = allow.unwrap_or_default();
            if let Some(exp) = expires {
                val.expires = exp;
            }
            val.allowance += amount;
            Ok(val)
        },
    )?;

    let res = Response::new().add_attributes(vec![
        attr("action", "increase_allowance"),
        attr("owner", info.sender),
        attr("spender", spender),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_decrease_allowance(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    if spender_addr == info.sender {
        return Err(ContractError::CannotSetOwnAccount {});
    }

    let key = (&info.sender, &spender_addr);
    // load value and delete if it hits 0, or update otherwise
    let mut allowance = ALLOWANCES.load(deps.storage, key)?;
    if amount < allowance.allowance {
        // update the new amount
        allowance.allowance = allowance
            .allowance
            .checked_sub(amount)
            .map_err(StdError::overflow)?;
        if let Some(exp) = expires {
            allowance.expires = exp;
        }
        ALLOWANCES.save(deps.storage, key, &allowance)?;
    } else {
        ALLOWANCES.remove(deps.storage, key);
    }

    let res = Response::new().add_attributes(vec![
        attr("action", "decrease_allowance"),
        attr("owner", info.sender),
        attr("spender", spender),
        attr("amount", amount),
    ]);
    Ok(res)
}

// this can be used to update a lower allowance - call bucket.update with proper keys
pub fn deduct_allowance(
    storage: &mut dyn Storage,
    owner: &Addr,
    spender: &Addr,
    block: &BlockInfo,
    amount: Uint128,
) -> Result<AllowanceResponse, ContractError> {
    ALLOWANCES.update(storage, (owner, spender), |current| {
        match current {
            Some(mut a) => {
                if a.expires.is_expired(block) {
                    Err(ContractError::Expired {})
                } else {
                    // deduct the allowance if enough
                    a.allowance = a
                        .allowance
                        .checked_sub(amount)
                        .map_err(StdError::overflow)?;
                    Ok(a)
                }
            }
            None => Err(ContractError::NoAllowance {}),
        }
    })
}

pub fn execute_transfer_from(
    deps: &mut DepsMut,
    env:  &mut Env,
    info: &mut MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new().add_attributes(vec![
        attr("action", "transfer_from"),
        attr("from", owner),
        attr("to", recipient),
        attr("by", info.sender.clone()),
        attr("amount", amount),
    ]);
    Ok(res)
}

pub fn execute_send_from(
    deps: &mut DepsMut,
    env: &mut Env,
    info: &mut MessageInfo,
    owner: String,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;
    let owner_addr = deps.api.addr_validate(&owner)?;

    // deduct allowance before doing anything else have enough allowance
    deduct_allowance(deps.storage, &owner_addr, &info.sender, &env.block, amount)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let attrs = vec![
        attr("action", "send_from"),
        attr("from", &owner),
        attr("to", &contract),
        attr("by", &info.sender),
        attr("amount", amount),
    ];

    // create a send message
    let msg = Cw20ReceiveMsg {
        sender: info.sender.clone().into(),
        amount,
        msg,
    }
    .into_cosmos_msg(contract)?;

    let res = Response::new().add_message(msg).add_attributes(attrs);
    Ok(res)
}

pub fn query_allowance(deps: Deps, owner: String, spender: String) -> StdResult<AllowanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = ALLOWANCES
        .may_load(deps.storage, (&owner_addr, &spender_addr))?
        .unwrap_or_default();
    Ok(allowance)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, CosmosMsg, SubMsg, Timestamp, WasmMsg};
    use cw20::{Cw20Coin, TokenInfoResponse};

    use crate::contract::{execute, instantiate, query_balance, query_token_info};
    use crate::msg::{ExecuteMsg, InstantiateMsg};

    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance
    }

    // this will set up the instantiation for other tests
    fn do_instantiate<T: Into<String>>(
        mut deps: DepsMut,
        addr: T,
        amount: Uint128,
    ) -> TokenInfoResponse {
        let instantiate_msg = InstantiateMsg {
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        query_token_info(deps.as_ref()).unwrap()
    }
}
