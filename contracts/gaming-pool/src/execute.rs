use std::ops::Add;

use astroport::asset::{Asset, AssetInfo};
use astroport::pair::ExecuteMsg as AstroPortExecute;
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env,
                   from_binary, MessageInfo, Order, Response, StdError,
                   StdResult, Storage, SubMsg, to_binary, Uint128, WasmMsg};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::contract::{CLAIMED_REFUND, CLAIMED_REWARD, DUMMY_WALLET, GAME_CANCELLED,
                      GAME_COMPLETED, GAME_POOL_CLOSED, GAME_POOL_OPEN, HUNDRED_PERCENT,
                      INITIAL_REFUND_AMOUNT, INITIAL_REWARD_AMOUNT, INITIAL_TEAM_POINTS,
                      INITIAL_TEAM_RANK, NINETY_NINE_NINE_PERCENT, REWARDS_DISTRIBUTED,
                      REWARDS_NOT_DISTRIBUTED, UNCLAIMED_REFUND, UNCLAIMED_REWARD};
use crate::ContractError;
use crate::msg::{ProxyQueryMsgs, QueryMsgSimulation, ReceivedMsg};
use crate::query::{get_team_count_for_user_in_pool_type, query_pool_details, query_pool_type_details};
use crate::state::{CONFIG, CONTRACT_POOL_COUNT, FeeDetails, GAME_DETAILS, GameDetails,
                   GameResult, PLATFORM_WALLET_PERCENTAGES, POOL_DETAILS,
                   POOL_TEAM_DETAILS, POOL_TYPE_DETAILS, PoolDetails, PoolTeamDetails,
                   PoolTypeDetails, WalletPercentage, WalletTransferDetails};

pub fn received_message(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceivedMsg = from_binary(&message.msg)?;
    let amount = Uint128::from(message.amount);
    match msg {
        ReceivedMsg::GamePoolBidSubmit(gpbsc) => game_pool_bid_submit(
            deps,
            env,
            info,
            gpbsc.gamer,
            gpbsc.pool_type,
            gpbsc.pool_id,
            gpbsc.team_id,
            amount,
            false,
        ),
    }
}

pub fn set_platform_fee_wallets(
    deps: DepsMut,
    info: MessageInfo,
    wallet_percentages: Vec<WalletPercentage>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }

    for wp in wallet_percentages {
        PLATFORM_WALLET_PERCENTAGES.save(
            deps.storage,
            wp.wallet_name.clone(),
            &WalletPercentage {
                wallet_name: wp.wallet_name.clone(),
                wallet_address: wp.wallet_address.clone(),
                percentage: wp.percentage,
            },
        )?;
    }
    return Ok(Response::default());
}

pub fn set_pool_type_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_type: String,
    pool_fee: Uint128,
    min_teams_for_pool: u32,
    max_teams_for_pool: u32,
    max_teams_for_gamer: u32,
    wallet_percentages: Vec<WalletPercentage>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let ptd = POOL_TYPE_DETAILS.may_load(deps.storage, pool_type.clone())?;
    match ptd {
        Some(_ptd) => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Pool type already set"),
            }));
        }
        None => {}
    };

    let mut rake_list: Vec<WalletPercentage> = Vec::new();
    for wp in wallet_percentages {
        rake_list.push(wp);
    }
    POOL_TYPE_DETAILS.save(
        deps.storage,
        pool_type.clone(),
        &PoolTypeDetails {
            pool_type: pool_type.clone(),
            pool_fee: pool_fee,
            min_teams_for_pool: min_teams_for_pool,
            max_teams_for_pool: max_teams_for_pool,
            max_teams_for_gamer: max_teams_for_gamer,
            rake_list: rake_list,
        },
    )?;
    return Ok(Response::default());
}

pub fn cancel_game(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let game_id = config.game_id;

    let gd = GAME_DETAILS.may_load(deps.storage, game_id.clone())?;
    let game;
    match gd {
        Some(gd) => {
            game = gd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Game status cannot be retrieved"),
            }));
        }
    }
    if game.game_status == GAME_COMPLETED {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Cant cancel game as it is already over"),
        }));
    }
    if game.game_status == GAME_CANCELLED {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Cant cancel game as it is already cancelled"),
        }));
    }

    GAME_DETAILS.save(
        deps.storage,
        game_id.clone(),
        &GameDetails {
            game_id: game_id.clone(),
            game_status: GAME_CANCELLED,
        },
    )?;

    // Get all pools
    let all_pools: Vec<String> = POOL_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for pool_id in all_pools {
        let mut pool;
        let pd = POOL_DETAILS.may_load(deps.storage, pool_id.clone())?;
        match pd {
            Some(pd) => {
                pool = pd;
            }
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No pool details found for pool"),
                }));
            }
        };
        let pool_type;
        let ptd = POOL_TYPE_DETAILS.may_load(deps.storage, pool.pool_type.clone())?;
        match ptd {
            Some(ptd) => {
                pool_type = ptd;
            }
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No pool type details found for pool"),
                }));
            }
        };
        let refund_amount = pool_type.pool_fee;
        pool.pool_refund_status = true; // We skip the iteration and update the status
        POOL_DETAILS.save(deps.storage, pool_id.clone(), &pool)?;
        // Get the existing teams for this pool
        // let mut teams = Vec::new();
        // let all_teams = POOL_TEAM_DETAILS.may_load(deps.storage, pool_id.clone())?;
        // match all_teams {
        //     Some(some_teams) => {
        //         let teams = some_teams;
        //         let mut updated_teams: Vec<PoolTeamDetails> = Vec::new();
        //         for team in teams {
        //             // No transfer to be done to the gamers. Just update their refund amounts.
        //             // They have to come and collect their refund
        //             // In case of refund due to lock_game min_team_count not met for the pool_type
        //             let mut updated_team = team.clone();
        //             if updated_team.refund_amount == Uint128::zero() {
        //                 updated_team.refund_amount = refund_amount;
        //                 updated_team.claimed_refund = UNCLAIMED_REFUND;
        //                 println!(
        //                     "refund for {:?} is {:?}",
        //                     team.team_id, updated_team.refund_amount
        //                 );
        //             }
        //             updated_teams.push(updated_team);
        //         }
        //         POOL_TEAM_DETAILS.save(deps.storage, pool_id.clone(), &updated_teams)?;
        //     }
        //     None => {}
        // }
    }
    return Ok(Response::new()
        .add_attribute("game_id", game_id.clone())
        .add_attribute("game_status", "GAME_CANCELLED".to_string()));
}

pub fn lock_game(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let game_id = config.game_id;

    let gd = GAME_DETAILS.may_load(deps.storage, game_id.clone())?;
    let game;
    match gd {
        Some(gd) => {
            game = gd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Game status cannot be retrieved"),
            }));
        }
    }
    if game.game_status != GAME_POOL_OPEN {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Cant lock this game as it is not open for bidding"),
        }));
    }

    GAME_DETAILS.save(
        deps.storage,
        game_id.clone(),
        &GameDetails {
            game_id: game_id.clone(),
            game_status: GAME_POOL_CLOSED,
        },
    )?;

    // Get all pools
    let all_pools: Vec<String> = POOL_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for pool_id in all_pools {
        let mut pool;
        let pd = POOL_DETAILS.may_load(deps.storage, pool_id.clone())?;

        match pd {
            Some(pd) => {
                pool = pd;
            }
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No pool details found for pool"),
                }));
            }
        };
        let pool_type;
        let ptd = POOL_TYPE_DETAILS.may_load(deps.storage, pool.pool_type.clone())?;
        match ptd {
            Some(ptd) => {
                pool_type = ptd;
            }
            None => {
                return Err(ContractError::Std(StdError::GenericErr {
                    msg: String::from("No pool type details found for pool"),
                }));
            }
        };
        if pool.current_teams_count >= pool_type.min_teams_for_pool {
            continue;
        }
        // let refund_amount = pool_type.pool_fee; //  Commented since we dont use this value anymore
        pool.pool_refund_status = true; // We skip the iteration and update the status
        POOL_DETAILS.save(deps.storage, pool_id.clone(), &pool)?;
        // COMMENTED CODE
        // Get the existing teams for this pool
        // let mut teams = Vec::new();
        // let all_teams = POOL_TEAM_DETAILS.may_load(deps.storage, pool_id.clone())?;
        // match all_teams {
        //     Some(some_teams) => {
        //         let teams = some_teams;
        //         let mut updated_teams: Vec<PoolTeamDetails> = Vec::new();
        //         for team in teams {
        //             // No transfer to be done to the gamers. Just update their refund amounts.
        //             // They have to come and collect their refund
        //             let mut updated_team = team.clone();
        //             updated_team.refund_amount = refund_amount;
        //             updated_team.claimed_refund = UNCLAIMED_REFUND;
        //             println!(
        //                 "refund for {:?} is {:?}",
        //                 team.team_id, updated_team.refund_amount
        //             );
        //             updated_teams.push(updated_team);
        //         }
        //         POOL_TEAM_DETAILS.save(deps.storage, pool_id.clone(), &updated_teams)?;
        //     }
        //     None => {}
        // }
    }
    return Ok(Response::new()
        .add_attribute("game_id", game_id.clone())
        .add_attribute("game_status", "GAME_POOL_CLOSED".to_string()));
}

pub fn create_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_type: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let game_id = config.game_id;
    let gd = GAME_DETAILS.may_load(deps.storage, game_id.clone())?;
    let game;
    match gd {
        Some(gd) => {
            game = gd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Game status cannot be retrieved"),
            }));
        }
    }
    if game.game_status != GAME_POOL_OPEN {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Game is not open for bidding"),
        }));
    }

    let dummy_wallet = String::from(DUMMY_WALLET);
    let address = deps.api.addr_validate(dummy_wallet.clone().as_str())?;
    let cpc = CONTRACT_POOL_COUNT.may_load(deps.storage, &address)?;
    let global_pool_id;
    match cpc {
        Some(cpc) => {
            global_pool_id = cpc;
        }
        None => {
            global_pool_id = Uint128::zero();
        }
    }
    let mut count = global_pool_id;
    CONTRACT_POOL_COUNT.update(
        deps.storage,
        &address,
        |global_pool_id: Option<Uint128>| -> StdResult<_> {
            Ok(global_pool_id.unwrap_or_default() + Uint128::from(1u128))
        },
    )?;
    count += Uint128::from(1u128);
    let pool_id_str: String = count.to_string();

    POOL_DETAILS.save(
        deps.storage,
        pool_id_str.clone(),
        &PoolDetails {
            game_id: game_id.clone(),
            pool_id: pool_id_str.clone(),
            pool_type: pool_type.clone(),
            current_teams_count: 0u32,
            rewards_distributed: REWARDS_NOT_DISTRIBUTED,
            pool_refund_status: false,
            pool_reward_status: false,
        },
    )?;
    return Ok(Response::new().add_attribute("pool_id", pool_id_str.clone()));
}

pub fn query_platform_fees(
    pool_fee: Uint128,
    platform_fees_percentage: Uint128,
    transaction_fee_percentage: Uint128,
) -> Result<FeeDetails, ContractError> {
    return Ok(FeeDetails {
        platform_fee: Uint128::from(pool_fee
            .checked_mul(platform_fees_percentage).unwrap()
            .checked_div(Uint128::from(HUNDRED_PERCENT)).unwrap()),
        transaction_fee: Uint128::from(pool_fee
            .checked_mul(transaction_fee_percentage).unwrap()
            .checked_div(Uint128::from(HUNDRED_PERCENT)).unwrap()),
    });
}

pub fn game_pool_bid_submit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    gamer: String,
    pool_type: String,
    pool_id: String,
    team_id: String,
    amount: Uint128,
    testing: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    // Calculate
    let platform_fee = config.platform_fee; //  Should be in %
    let game_id = config.clone().game_id;
    let mut messages = Vec::new(); //  Use this to append any execute messaages in the funciton
    let gd = GAME_DETAILS.may_load(deps.storage, game_id.clone())?;
    let game;
    match gd {
        Some(gd) => {
            game = gd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Game status cannot be retrieved"),
            }));
        }
    }
    if game.game_status != GAME_POOL_OPEN {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Game is not open for bidding"),
        }));
    }

    let pool_type_details;
    let ptd = POOL_TYPE_DETAILS.may_load(deps.storage, pool_type.clone())?;
    match ptd.clone() {
        Some(ptd) => {
            pool_type_details = ptd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Cant get details for pool type "),
            }));
        }
    }
    let required_platform_fee_ust;
    let transaction_fee;
    match testing {
        true => {
            required_platform_fee_ust = Uint128::zero();
            transaction_fee = Uint128::zero();
        }
        false => {
            let fee_details = query_platform_fees(
                ptd.unwrap().pool_fee,
                platform_fee,
                config.transaction_fee,
            )?;
            required_platform_fee_ust = fee_details.platform_fee;
            transaction_fee = fee_details.transaction_fee;
        }
    }

    if !testing {
        let mut asset: Asset = Asset {
            info: AssetInfo::NativeToken { denom: info.funds[0].denom.clone() },
            amount: info.funds[0].amount,
        };
        if info.funds.clone().len() != 1 {
            return Err(ContractError::InvalidNumberOfCoinsSent {});
        }
        let fund = info.funds.clone();

        if fund[0].denom == "uusd" {
            if fund[0].amount < required_platform_fee_ust.add(transaction_fee) {
                asset = Asset {
                    info: AssetInfo::NativeToken { denom: fund[0].denom.clone() },
                    amount: fund[0].amount,
                };
                println!("Asset {}", asset);
            }
        } else {
            return Err(ContractError::InsufficientFeesUst {});
        }
        println!("Asset {}", asset);
    }


    let mut pool_fee: Uint128 = pool_type_details.pool_fee;
    if !testing {
        pool_fee = deps.querier.query_wasm_smart(
            config.clone().astro_proxy_address,
            &ProxyQueryMsgs::get_fury_equivalent_to_ust {
                ust_count: pool_type_details.pool_fee,
            },
        )?;
    }

    // let platform_fee = pool_fee
    //     .checked_mul(platform_fee)?;
    // let transaction_fee = pool_fee.checked_mul(config.transaction_fee)?;
    let max_teams_for_pool = pool_type_details.max_teams_for_pool;
    let max_teams_for_gamer = pool_type_details.max_teams_for_gamer;
    let amount_required = pool_fee
        * (Uint128::from(NINETY_NINE_NINE_PERCENT))
        / (Uint128::from(HUNDRED_PERCENT));
    if amount < amount_required {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Amount being bid does not match the pool fee and the platform fee"),
        }));
    }
    let mut user_team_count = 0;
    // Here we load the details based on the user placing the bid
    let ptd = POOL_TEAM_DETAILS.may_load(deps.storage, (&pool_id.clone(), info.sender.as_ref()))?;
    match ptd {
        Some(std) => {
            let all_teams = std;
            for team in all_teams {
                if team.gamer_address == gamer {
                    user_team_count += 1;
                }
            }
        }
        None => {}
    }
    println!("user team count = {:?}", user_team_count);
    if user_team_count >= max_teams_for_gamer {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("User max team limit reached "),
        }));
    }

    let pool_id_return;
    let mut pool_details = query_pool_details(deps.storage, pool_id.clone())?;

    // check if the pool can accomodate the team
    if pool_details.current_teams_count < max_teams_for_pool {
        pool_id_return = pool_id.clone();
        pool_details.current_teams_count += 1;
        POOL_DETAILS.save(
            deps.storage,
            pool_id.clone(),
            &PoolDetails {
                pool_type: pool_type.clone(),
                pool_id: pool_id.clone(),
                game_id: pool_details.game_id.clone(),
                current_teams_count: pool_details.current_teams_count,
                rewards_distributed: pool_details.rewards_distributed,
                pool_refund_status: false,
                pool_reward_status: false,
            },
        )?;
        // Now save the team details
        save_team_details(
            deps.storage,
            env.clone(),
            gamer.clone(),
            pool_id.clone(),
            team_id.clone(),
            game_id.clone(),
            pool_type.clone(),
            Uint128::from(INITIAL_REWARD_AMOUNT),
            UNCLAIMED_REWARD,
            Uint128::from(INITIAL_REFUND_AMOUNT),
            UNCLAIMED_REFUND,
            INITIAL_TEAM_POINTS,
            INITIAL_TEAM_RANK,
        )?;
    } else {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("pool max team limit reached "),
        }));
    }

    // Sending Fury token to the contract
    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: info.sender.into_string(),
        recipient: env.clone().contract.address.to_string(),
        amount,
    };
    let exec = WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    };
    messages.push(CosmosMsg::Wasm(exec));


    let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: String::from(config.clone().astro_proxy_address),
        amount,
        expires: None,
    };
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.clone().minting_contract_address.to_string(),
        msg: to_binary(&increase_allowance_msg).unwrap(),
        funds: vec![],
    }));


    let fury_asset_info = Asset {
        info: AssetInfo::Token {
            contract_addr: config.clone().minting_contract_address.clone()
        },
        amount,
    };
    let swap_message = AstroPortExecute::Swap {
        offer_asset: fury_asset_info,
        belief_price: None,
        max_spread: Option::from(Decimal::from("0.5".to_string().parse().unwrap())),
        to: Option::from(env.contract.address.to_string()),
    };
    // let tax_in_fury = fury_asset_info.deduct_tax(&deps.querier)?;
    let platform_fees = deps.querier.query_wasm_smart(
        config.clone().astro_proxy_address,
        &QueryMsgSimulation::QueryPlatformFees {
            msg: to_binary(&swap_message)?
        },
    )?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.clone().astro_proxy_address.to_string(),
        msg: to_binary(&swap_message).unwrap(),
        funds: vec![Coin {
            denom: "uusd".to_string(),
            amount: platform_fees,
        }],
    }));
    return Ok(Response::new()
        .add_attribute("pool_id", pool_id_return.clone())
        .add_messages(messages));
}

pub fn save_team_details(
    storage: &mut dyn Storage,
    _env: Env,
    gamer: String,
    pool_id: String,
    team_id: String,
    game_id: String,
    pool_type: String,
    reward_amount: Uint128,
    claimed_reward: bool,
    refund_amount: Uint128,
    claimed_refund: bool,
    team_points: u64,
    team_rank: u64,
) -> Result<Response, ContractError> {
    // Get the existing teams for this pool
    let mut teams = Vec::new();
    let all_teams = POOL_TEAM_DETAILS.may_load(storage, (&pool_id.clone(), gamer.clone().as_ref()))?;
    match all_teams {
        Some(some_teams) => {
            teams = some_teams;
        }
        None => {}
    }

    teams.push(PoolTeamDetails {
        gamer_address: gamer.clone(),
        game_id: game_id.clone(),
        pool_type: pool_type.clone(),
        pool_id: pool_id.clone(),
        team_id: team_id.clone(),
        reward_amount,
        claimed_reward,
        refund_amount,
        claimed_refund,
        team_points,
        team_rank,
    });
    POOL_TEAM_DETAILS.save(storage, (&pool_id.clone(), gamer.as_ref()), &teams)?;

    return Ok(Response::new().add_attribute("team_id", team_id.clone()));
}

// Reward:Platform fee has to charged. Reward amount here is in FURY.
// Make a call to astroport to get the platform fee, that is to be charged.
// Here we only transfer the FURY and here since the amount is in
// FURY no swap needs to be done so no call to astroport for swap.
pub fn claim_reward(
    deps: DepsMut,
    info: MessageInfo,
    gamer: String,
    env: Env,
) -> Result<Response, ContractError> {
    let gamer_addr = deps.api.addr_validate(&gamer)?;
    //Check if withdrawer is same as invoker
    if gamer_addr != info.sender {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }

    let mut user_reward = Uint128::zero();
    // Get all pools
    let all_pools: Vec<String> = POOL_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    for pool_id in all_pools {
        // Get the existing teams for this pool
        let mut pool_details: PoolDetails = Default::default();
        let pd = POOL_DETAILS.load(deps.storage, pool_id.clone());
        match pd {
            Ok(some) => { pool_details = some; }
            Err(_) => {
                continue;
            }
        }
        if !pool_details.pool_reward_status {
            continue;
        }
        let mut pool_team_details;
        match POOL_TEAM_DETAILS.load(deps.storage, (&*pool_id.clone(), info.sender.as_ref())) {
            Ok(some) => { pool_team_details = some }
            Err(_) => {
                continue;
            }
        }
        let mut updated_details = Vec::new();
        for team_details in pool_team_details {
            if !team_details.claimed_reward {
                let mut updated_team = team_details.clone();
                user_reward += team_details.reward_amount;
                updated_team.claimed_reward = true;
                updated_details.push(updated_team);
            } else {
                updated_details.push(team_details);
            }
        }
        if !updated_details.is_empty() {
            POOL_TEAM_DETAILS.save(deps.storage, (&*pool_id, info.sender.as_ref()), &updated_details)?
        }
    }

    // let mut teams = Vec::new();
    //   let all_teams = POOL_TEAM_DETAILS.may_load(deps.storage, (&*pool_id.clone(), info.sender.as_ref()))?;
    //   match all_teams {
    //       Some(some_teams) => {
    //           teams = some_teams;
    //       }
    //       None => {}
    //   }
    //
    //   let existing_teams = teams.clone();
    //   let mut updated_teams = Vec::new();
    //   for team in existing_teams {
    //       let mut updated_team = team.clone();
    //       println!("Gamer {:?} gamer_address {:?} ", gamer, team.gamer_address);
    //       if gamer == team.gamer_address && team.claimed_reward == UNCLAIMED_REWARD {
    //           user_reward += team.reward_amount;
    //           updated_team.claimed_reward = CLAIMED_REWARD;
    //       }
    //       updated_teams.push(updated_team);
    //   }
    //   POOL_TEAM_DETAILS.save(deps.storage, (&*pool_id.clone(), info.sender.as_ref()), &updated_teams)?;

    println!("reward amount is {:?}", user_reward);

    if user_reward == Uint128::zero() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("No reward for this user"),
        }));
    }

    // Do the transfer of reward to the actual gamer_addr from the contract
    let config = CONFIG.load(deps.storage)?;
    let mut messages = Vec::new();
    let fee_details = query_platform_fees(user_reward, config.platform_fee, config.transaction_fee)?;
    // We only take the first coin object since we only expect UST here
    let funds_sent = info.funds[0].clone();
    if (funds_sent.denom != "uusd") || (funds_sent.amount < fee_details.platform_fee.add(fee_details.transaction_fee)) {
        return Err(ContractError::InsufficientFeesUst {});
    }
    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: env.clone().contract.address.to_string(),
        recipient: info.sender.into_string(),
        amount: user_reward,
    };
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.minting_contract_address.to_string(),
        msg: to_binary(&transfer_msg).unwrap(),
        funds: vec![],
    }));
    return Ok(Response::new()
        .add_attribute("amount", user_reward.to_string())
        .add_attribute("action", "reward")
        .add_messages(messages)
    );
}

// Refund: Pool fee is in UST but has to be given back in FURY,
// It is 10UST Equivant of Fury, Use Query platform fee on UST value directly.
// This means it has to be swapped. So we make a call to astorport
// to swap it and we also need to pass the swap fee.
// No Platform fee charged at time of refund, we only
// refund the fee and swap fee is accepted by the contract.
// Transafer of UST and FURY has to be done together at refund.
pub fn claim_refund(
    deps: DepsMut,
    info: MessageInfo,
    gamer: String,
    env: Env,
    testing: Option<bool>,
) -> Result<Response, ContractError> {
    let testing_status = testing.unwrap_or(false);
    let mut refund_in_ust_fees = Uint128::default();
    let gamer_addr = deps.api.addr_validate(&gamer)?;
    //Check if withdrawer is same as invoker
    if gamer_addr != info.sender {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let config = CONFIG.load(deps.storage)?;
    // Get all pools

    let all_pools: Vec<String> = POOL_DETAILS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    let mut total_refund_amount = Uint128::zero();
    for pool_id in all_pools {
        let mut pool_details: PoolDetails = Default::default();
        let pd = POOL_DETAILS.load(deps.storage, pool_id.clone());
        match pd {
            Ok(some) => { pool_details = some; }
            Err(_) => {
                continue;
            }
        }
        if !pool_details.pool_refund_status {
            continue;
        }
        let pool_type = POOL_TYPE_DETAILS.load(deps.storage, pool_details.pool_type)?;
        let refund_amount = pool_type.pool_fee;
        let pool_team_details = POOL_TEAM_DETAILS.load(deps.storage, (pool_id.as_ref(), info.sender.as_ref()))?.clone();
        let mut updated_details = Vec::new();
        for team_details in pool_team_details {
            if !team_details.claimed_refund {
                let mut updated_team = team_details.clone();
                updated_team.refund_amount = refund_amount;
                total_refund_amount += refund_amount;
                updated_team.claimed_refund = true;
                updated_details.push(updated_team);
            } else {
                return Err(ContractError::RefundAlreadyClaimed {});
            }
        }
        if !updated_details.is_empty() {
            POOL_TEAM_DETAILS.save(deps.storage, (pool_id.as_ref(), info.sender.as_ref()), &updated_details)?
        }
        // Get the existing teams for this pool
        // let mut teams = Vec::new();
        // let all_teams = POOL_TEAM_DETAILS.may_load(deps.storage, (&*pool_id.clone(), info.sender.as_ref()))?;
        // match all_teams {
        //     Some(some_teams) => {
        //         teams = some_teams;
        //     }
        //     None => {}
        // }
        //
        // let existing_teams = teams.clone();
        // let mut updated_teams = Vec::new();
        //
        // for team in existing_teams {
        //     let mut updated_team = team.clone();
        //     println!("Gamer {:?} gamer_address {:?} ", gamer, team.gamer_address);
        //     if gamer == team.gamer_address && team.claimed_refund == UNCLAIMED_REFUND {
        //         user_refund += team.refund_amount;
        //         updated_team.claimed_refund = CLAIMED_REFUND;
        //         let pool_details = query_pool_type_details(deps.storage, team.pool_type)?;
        //         let refund_details = query_platform_fees(pool_details.pool_fee, config.platform_fee, config.transaction_fee)?;
        //         refund_in_ust_fees += refund_details.transaction_fee.add(refund_details.platform_fee);
        //     }
        //     updated_teams.push(updated_team);
        // }
        // POOL_TEAM_DETAILS.save(deps.storage, (&*pool_id.clone(), info.sender.as_ref()), &updated_teams)?;
    }

    println!("refund amount is {:?}", total_refund_amount);

    if total_refund_amount == Uint128::zero() {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("No refund for this user"),
        }));
    }
    let refund_details = query_platform_fees(total_refund_amount, config.platform_fee, config.transaction_fee)?;
    refund_in_ust_fees = refund_details.transaction_fee.add(refund_details.platform_fee);
    // Do the transfer of refund to the actual gamer_addr from the contract
    let mut messages = Vec::new();
    let ust_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string()
        },
        amount: total_refund_amount,
    };
    let tax = ust_asset.compute_tax(&deps.querier)?;
    // ust_asset.amount += tax;
    let swap_message = AstroPortExecute::Swap {
        offer_asset: ust_asset.clone(),
        belief_price: None,
        max_spread: None,
        to: Option::from(info.sender.to_string()),
    };

    let mut swap_fee = Uint128::zero();
    // Swap fee should be platform+transaction fee for the transaction
    if !testing_status {
        swap_fee = deps.querier.query_wasm_smart(
            config.clone().astro_proxy_address,
            &QueryMsgSimulation::QueryPlatformFees {
                msg: to_binary(&swap_message)?
            },
        )?;
    }
    let final_amount = ust_asset.amount.clone().add(swap_fee).add(tax);
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.astro_proxy_address.to_string(),
        msg: to_binary(&swap_message)?,
        funds: vec![Coin {
            denom: "uusd".to_string(),
            amount: final_amount,
        }],
    }));
    let refund = Coin {
        denom: "uusd".to_string(),
        amount: refund_in_ust_fees,
    };
    let mut refund_: Vec<Coin> = vec![];
    refund_.push(refund);
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: String::from(info.sender),
        amount: refund_,
    }));
    return Ok(Response::new()
        .add_attribute("amount", final_amount.to_string())
        .add_attribute("action", "refund")
        .add_messages(messages)
    );
}

pub fn game_pool_reward_distribute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_id: String,
    game_winners: Vec<GameResult>,
    is_final_batch: bool,
    testing: bool,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let platform_fee_in_percentage = config.platform_fee;
    let platform_fee;
    let game_id = config.game_id;

    let gd = GAME_DETAILS.may_load(deps.storage, game_id.clone())?;
    let game;
    match gd {
        Some(gd) => {
            game = gd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Game status cannot be retrieved"),
            }));
        }
    }
    if game.game_status == GAME_CANCELLED {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Rewards cant be distributed as game is cancelled"),
        }));
    }
    if game.game_status == GAME_POOL_OPEN {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Rewards cant be distributed as game not yet started"),
        }));
    }
    let reward_status;
    let game_status;
    let pool_status_string;
    let reward_status_string;
    if is_final_batch {
        reward_status = true;
        game_status = GAME_COMPLETED;
        reward_status_string = "GAME_COMPLETED";
        pool_status_string = "POOL_REWARD_DISTRIBUTED";
    } else {
        reward_status_string = "GAME_NOT_COMPLETED";
        pool_status_string = "POOL_REWARD_DISTRIBUTED_INCOMPLETE";
        reward_status = false;
        game_status = GAME_POOL_OPEN
    }
    GAME_DETAILS.save(
        deps.storage,
        game_id.clone(),
        &GameDetails {
            game_id: game_id.clone(),
            game_status: game_status,
        },
    )?;

    let pool_details = query_pool_details(deps.storage, pool_id.clone())?;
    if pool_details.rewards_distributed == REWARDS_DISTRIBUTED {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("Rewards are already distributed for this pool"),
        }));
    }
    let pool_count = pool_details.current_teams_count;
    let pool_type = pool_details.pool_type;


    POOL_DETAILS.save(
        deps.storage,
        pool_id.clone(),
        &PoolDetails {
            game_id: game_id.clone(),
            pool_id: pool_id.clone(),
            pool_type: pool_type.clone(),
            current_teams_count: pool_details.current_teams_count,
            rewards_distributed: reward_status,
            pool_refund_status: false,
            pool_reward_status: true,
        },
    )?;

    let pool_type_details;
    let ptd = POOL_TYPE_DETAILS.may_load(deps.storage, pool_type.clone())?;
    match ptd {
        Some(ptd) => {
            pool_type_details = ptd;
        }
        None => {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: String::from("Cant get details for pool type"),
            }));
        }
    }
    platform_fee = query_platform_fees(pool_type_details.pool_fee, platform_fee_in_percentage, config.transaction_fee.clone())?.platform_fee;


    // let pool_fee: Uint128 = deps.querier.query_wasm_smart(
    //     config.astro_proxy_address,
    //     &ProxyQueryMsgs::get_fury_equivalent_to_ust {
    //         ust_count: pool_type_details.pool_fee,
    //     },
    // )?;

    let pool_fee: Uint128 = pool_type_details.pool_fee;

    let total_reward = pool_fee
        .checked_mul(Uint128::from(pool_count))
        .unwrap_or_default();

    let mut winner_rewards = Uint128::zero();
    let winners = game_winners.clone();
    for winner in winners {
        winner_rewards += winner.reward_amount;
    }
    if total_reward <= winner_rewards {
        return Err(ContractError::Std(StdError::GenericErr {
            msg: String::from("reward amounts do not match"),
        }));
    }

    let rake_amount = total_reward - winner_rewards;
    println!(
        "total_reward {:?} winner_rewards {:?} rake_amount {:?}",
        total_reward, winner_rewards, rake_amount
    );

    let mut wallet_transfer_details: Vec<WalletTransferDetails> = Vec::new();

    let total_platform_fee = platform_fee
        .checked_mul(Uint128::from(pool_count))
        .unwrap_or_default();
    // Transfer total_platform_fee to platform wallets
    // These are the refund and development wallets
    let all_wallet_names: Vec<String> = PLATFORM_WALLET_PERCENTAGES
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|k| String::from_utf8(k).unwrap())
        .collect();
    let mut total_transfer_amount_in_fury = Uint128::zero();
    for wallet_name in all_wallet_names {
        let wallet = PLATFORM_WALLET_PERCENTAGES.load(deps.storage, wallet_name.clone())?;
        let wallet_address = wallet.wallet_address;
        let proportionate_amount = total_platform_fee
            .checked_mul(Uint128::from(wallet.percentage))
            .unwrap_or_default()
            .checked_div(Uint128::from(100u128))
            .unwrap_or_default();
        let transfer_detail = WalletTransferDetails {
            wallet_address: wallet_address.clone(),
            amount: proportionate_amount,
        };
        total_transfer_amount_in_fury += proportionate_amount;
        wallet_transfer_details.push(transfer_detail);
        println!(
            "transferring {:?} to {:?}",
            proportionate_amount,
            wallet_address.clone()
        );
    }

    // Get all teams for this pool
    let mut reward_given_so_far = Uint128::zero();
    let mut all_teams: Vec<PoolTeamDetails> = Vec::new();
    for winner in game_winners.clone().into_iter() {
        let ptd = POOL_TEAM_DETAILS.may_load(deps.storage, (&pool_id.clone(), winner.gamer_address.as_ref()))?;
        match ptd {
            Some(ptd) => {
                all_teams = ptd;
            }
            None => {
                continue;
            }
        }
        let mut updated_teams: Vec<PoolTeamDetails> = Vec::new();
        for team in &all_teams {
            // No transfer to be done to the winners. Just update their reward amounts.
            // They have to come and collect their rewards
            let mut updated_team = team.clone();
            let winners = game_winners.clone();
            for winner in winners {
                if team.gamer_address == winner.gamer_address
                    && team.team_id == winner.team_id
                    && team.game_id == winner.game_id
                {
                    updated_team.reward_amount = winner.reward_amount;
                    updated_team.team_rank = winner.team_rank;
                    updated_team.team_points = winner.team_points;
                    reward_given_so_far += winner.reward_amount;
                    println!(
                        "reward for {:?} is {:?}",
                        team.team_id, updated_team.reward_amount
                    );
                }
            }
            updated_teams.push(updated_team);
        }
        POOL_TEAM_DETAILS.save(deps.storage, (&pool_id.clone(), winner.gamer_address.as_ref()), &updated_teams)?;
    }


    // Transfer rake_amount to all the rake wallets. Can also be only one rake wallet
    for wallet in pool_type_details.rake_list {
        let wallet_address = wallet.wallet_address;
        let proportionate_amount = rake_amount
            .checked_mul(Uint128::from(wallet.percentage))
            .unwrap_or_default()
            .checked_div(Uint128::from(100u128))
            .unwrap_or_default();
        // Transfer proportionate_amount to the corresponding rake wallet
        let transfer_detail = WalletTransferDetails {
            wallet_address: wallet_address.clone(),
            amount: proportionate_amount,
        };
        wallet_transfer_details.push(transfer_detail);
        println!(
            "transferring {:?} to {:?}",
            proportionate_amount,
            wallet_address.clone()
        );
    }

    let rsp = _transfer_to_multiple_wallets(
        wallet_transfer_details,
        "rake_and_platform_fee".to_string(),
        deps,
        testing,
    )?;
    return Ok(rsp
        .add_attribute("game_status", reward_status_string.to_string())
        .add_attribute("game_id", game_id.clone())
        .add_attribute("pool_status", pool_status_string.to_string())
        .add_attribute("pool_id", pool_id.clone()));
}

pub fn _transfer_to_multiple_wallets(
    wallet_details: Vec<WalletTransferDetails>,
    action: String,
    deps: DepsMut,
    testing: bool,
) -> Result<Response, ContractError> {
    let mut rsp = Response::new();
    if testing {
        return Ok(rsp);
    }
    for wallet in wallet_details {
        let current_amt = wallet.amount;
        let r = CosmosMsg::Bank(BankMsg::Send {
            to_address: wallet.wallet_address,
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: current_amt,
            }],
        });
        let send: SubMsg = SubMsg::new(r);
        rsp = rsp.add_submessage(send);
    }
    let data_msg = format!("Amount transferred").into_bytes();
    Ok(rsp.add_attribute("action", action).set_data(data_msg))
}

pub fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin_address {
        return Err(ContractError::Unauthorized {
            invoker: info.sender.to_string(),
        });
    }
    let ust_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: "uusd".to_string()
        },
        amount,
    };
    let tax = ust_asset.compute_tax(&deps.querier)?;
    let swap_message = AstroPortExecute::Swap {
        offer_asset: ust_asset.clone(),
        belief_price: None,
        max_spread: None,
        to: Option::from(env.contract.address.to_string()),
    };

    // Swap fee should be platform+transaction fee for the transaction
    let swap_fee: Uint128 = deps.querier.query_wasm_smart(
        config.clone().astro_proxy_address,
        &QueryMsgSimulation::QueryPlatformFees {
            msg: to_binary(&swap_message)?
        },
    )?;
    let final_amount = ust_asset.amount.clone().add(swap_fee).add(tax);
    return Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.astro_proxy_address.to_string(),
        msg: to_binary(&swap_message)?,
        funds: vec![Coin {
            denom: "uusd".to_string(),
            amount: final_amount,
        }],
    })));
}

pub fn execute_sweep(
    deps: DepsMut,
    info: MessageInfo,
    funds_to_send: Vec<Coin>) -> Result<Response, ContractError> {
    let state = CONFIG.load(deps.storage)?;

    if info.sender != state.admin_address {
        return Err(ContractError::Unauthorized { invoker: info.sender.clone().to_string() });
    }
    let r = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: funds_to_send,
    });
    Ok(Response::new()
        .add_message(r)
        .add_attribute("action", "exeucte_sweep"))
}
