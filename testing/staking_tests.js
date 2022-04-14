import {MnemonicKey, MsgSend} from "@terra-money/terra.js";
import {mint_wallet, terraClient, walletTest1} from "./constants.js";
import {executeContract, instantiateContract, queryContract, storeCode} from "./utils.js";
import {readFile} from 'fs/promises';
import {sleep} from "./migrate.js";
import {transferFuryTokens} from "./gaming.js";


/*
This script is meant to be used as a boiler-plate for setting up load testing .
Here we can perform the initial setup of the contract workflow and then send
the main endpoint for bulk_testing to execute_from_all
*/
//This is the wallet that will fund all the new wallets
const funding_wallet = walletTest1

let proxy = "terra1dazgw2z5sxe7hgt43p0e3xyljnu45tlzwraccz"
let furyContractAddress = "terra1sfga5c35trjwvgpfz8r7mh0zfecs3y2flf4khl"
let club_staking_address = null
let clubStakingInitMessage = {
    admin_address: walletTest1.key.accAddress,
    minting_contract_address: furyContractAddress,
    astro_proxy_address: proxy,
    platform_fees_collector_wallet: walletTest1.key.accAddress,
    club_fee_collector_wallet: walletTest1.key.accAddress,
    club_reward_next_timestamp: "1640447808000000000",
    reward_periodicity: 300,
    club_price: "100000",
    bonding_duration: 120,
    platform_fees: "100",
    transaction_fees: "30",
    control_fees: "50",
    max_bonding_limit_per_user: 100,
    owner_release_locking_duration: 0
}

async function increaseRewardAmount(amount) {
    // Here IRArequest needs to be called one and distribute reward needs to be called 100x
    let iraRequest = {
        increase_reward_amount: {
            reward_from: mint_wallet.key.accAddress
        }
    };
    let msgString = Buffer.from(JSON.stringify(iraRequest)).toString('base64');

    let viaMsg = {
        send: {
            contract: club_staking_address,
            amount: `${amount}`,
            msg: msgString
        }
    };
    console.log(viaMsg)

    let iraResponse = await executeContract(mint_wallet, furyContractAddress, viaMsg);
    console.log(`Increase Reward Amount Response ${iraResponse.txhash}`)
}


function bankTransferFund(wallet_from, wallet_to, uusd_amount) {
    console.log(`Funding ${wallet_to.key.accAddress}`);
    return new Promise(resolve => {
        // create a simple message that moves coin balances
        const send1 = new MsgSend(
            wallet_from.key.accAddress,
            wallet_to.key.accAddress,
            {uusd: uusd_amount}
        );

        wallet_from
            .createAndSignTx({
                msgs: [send1],
                memo: 'Initial Funding!',
            })
            .then(tx => terraClient.tx.broadcast(tx))
            .then(result => {
                console.log(result.txhash);
                resolve(result.txhash);
            });
    })
}


const wallets_json = JSON.parse(
    await readFile(
        new URL('wallets.json', import.meta.url)
    )
);

export async function load_funds(wallets, ust_funds, fury_funds) {
    let promises_to_fulfill = []
    for (const wallet of wallets) {
        await bankTransferFund(funding_wallet, wallet, `${ust_funds}000000`)
        await transferFuryTokens(wallet.key.accAddress, `${fury_funds}000000`)
    }
}

async function query_club_staking(club_name, users) {
    console.log(`Querying Club staking for ${users.length} users`)
    let batch = []
    for (let i = 0; i < users.length; i++) {
        let user = users[i]
        batch.push(user.key.accAddress)
        if (batch.length > 4) {
            let query = {
                club_staking_details: {
                    club_name: club_name,
                    user_list: batch
                }
            }
            let response = await queryContract(club_staking_address, query)
            console.log(response)
            batch = []
        }

    }
    let query = {
        club_staking_details: {
            club_name: club_name,
            user_list: batch
        }
    }
    let response = await queryContract(club_staking_address, query)
    console.log(response)

}

async function distribute_reward_perbatch(club_name, users, is_first_def = true) {
    let is_first_batch = is_first_def //  We set this to false on exec
    let batch = []
    let i = 0
    for (let j = 0; i < users.length; j++) {
        i += 1
        let user = users[j]
        let limit_perbatch = (is_first_batch) ? 1 : 2;
        batch.push(user.key.accAddress)
        if (batch.length >= limit_perbatch && is_first_batch) {
            await executeContract(walletTest1, club_staking_address, {
                calculate_and_distribute_rewards: {
                    "staker_list": batch,
                    "club_name": club_name,
                    "is_first_batch": is_first_batch,
                    "is_final_batch": false
                }
            })
            batch = []
            is_first_batch = false

        }
        if (batch.length >= limit_perbatch) {
            let is_last_batch = ((i >= users.length));
            await executeContract(walletTest1, club_staking_address, {
                calculate_and_distribute_rewards: {
                    "staker_list": batch,
                    "club_name": club_name,
                    "is_first_batch": is_first_batch,
                    "is_final_batch": is_last_batch
                }
            })
            batch = []
        }
    }
    if (batch.length > 0) {
        let response = await executeContract(walletTest1, club_staking_address, {
            calculate_and_distribute_rewards: {
                "staker_list": batch,
                "club_name": club_name,
                "is_first_batch": is_first_batch,
                "is_final_batch": true
            }
        })
        console.log(response)
    }


}

export function wallets_to_obj(wallets) {
    console.log("Test Initiated")
    let wallet_objects = []
    for (const wallet in wallets) {
        const mk = new MnemonicKey({mnemonic: wallet});
        wallet_objects.push(terraClient.wallet(mk))
        if (wallet_objects.length > 5) {
            break
        }
    }
    console.log("Wallets Ready....")
    return wallet_objects
}

async function stake_from_all(club_name, wallets) {
    for (const wallet of wallets) {
        await stakeOnAClub(wallet, club_name)
    }
}

const deploy_contract = async function (file, init) {
    console.log("Deploying Contract")
    console.log(init)
    const contractId = await storeCode(walletTest1, file,)
    await sleep(2100)
    const gamingInit = await instantiateContract(walletTest1, contractId, init)
    console.log(`New Contract Init Hash ${gamingInit.txhash}`)
    return gamingInit.logs[0].events[0].attributes[3].value; // Careful with order of argument
}

const fund = async function (wallet) {
    console.log("Sending 5k fury Tokens from Minter to gamer")
    let response = await transferFuryTokens(wallet, "5000000000")
    await bankTransferFund(funding_wallet, wallet, 100)
}
const buy_a_club = async function (user, club_name) {
    console.log("Performing Buy a club, increasing allowance")
    let increaseAllowanceMsg = {
        increase_allowance: {
            spender: club_staking_address,
            amount: "100000"
        }
    };
    let incrAllowResp = await executeContract(user, furyContractAddress, increaseAllowanceMsg);
    console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);
    let bacRequest = {
        buy_a_club: {
            buyer: user.key.accAddress,
            club_name: club_name,
            auto_stake: true
        }
    };
    let platformFees = await queryContract(club_staking_address, {query_platform_fees: {msg: Buffer.from(JSON.stringify(bacRequest)).toString('base64')}});
    console.log(`platformFees = ${JSON.stringify(platformFees)}`);
    let bacResponse = await executeContract(user, club_staking_address, bacRequest, {'uusd': Number(platformFees)});
    console.log("Buy a club transaction hash = " + bacResponse['txhash']);


}

async function stakeOnAClub(wallet, club_name) {
    //let Sameer stakeOn a club
    // first increase allowance for club staking contract on Sameer wallet to let it move fury
    let increaseAllowanceMsg = {
        increase_allowance: {
            spender: club_staking_address,
            amount: "100000"
        }
    };
    let incrAllowResp = await executeContract(wallet, furyContractAddress, increaseAllowanceMsg);
    console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);

    let soacRequest = {
        stake_on_a_club: {
            staker: wallet.key.accAddress,
            club_name: club_name,
            amount: "100000",
            auto_stake: true,
        }
    };
    let platformFees = await queryContract(club_staking_address, {query_platform_fees: {msg: Buffer.from(JSON.stringify(soacRequest)).toString('base64')}});
    console.log(`platformFees = ${JSON.stringify(platformFees)}`);
    let soacResponse = await executeContract(wallet, club_staking_address, soacRequest, {'uusd': Number(platformFees)});
    console.log("Stake on a club transaction hash = " + soacResponse['txhash']);
    await sleep(100)
}


//----------------------------------------- TESTING ---------------------------------------------------------------
// Inital Setup
export let wallets_for_test = await wallets_to_obj(wallets_json)
await load_funds(wallets_for_test, 10, 100)
// // Run the contract setup and deployment
// club_staking_address = await deploy_contract(ClubStakingContractPath, clubStakingInitMessage)
// console.log(`Cubstaking Address:${club_staking_address}`)
// //Fund the wallets
// await fund(nitin_wallet)
// await fund(sameer_wallet)
// // Buy a club
// await buy_a_club(nitin_wallet, "clubA")
// await buy_a_club(sameer_wallet, "clubB")
// //-----------------------
// // Now 5k
// await stake_from_all("clubA", wallets_for_test)
// await stake_from_all("clubB", wallets_for_test)
// wallets_for_test.push(nitin_wallet)
// wallets_for_test.push(sameer_wallet)
// // Query stakes
// await query_club_staking("clubA", wallets_for_test)
// await query_club_staking("clubB", wallets_for_test)
// // Increase Reward Amount
// await increaseRewardAmount(1000000)
// // Distribute Rewards
// await distribute_reward_perbatch("clubA", wallets_for_test)
// await distribute_reward_perbatch("clubB", wallets_for_test, false)
// // Query stakes Again to check if balances have increased
// await query_club_staking("clubA", wallets_for_test)
// await query_club_staking("clubB", wallets_for_test)