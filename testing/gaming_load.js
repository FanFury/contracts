import {GamingContractPath, sleep_time, terraClient, walletTest1} from './constants.js';
import {executeContract, instantiateContract, queryContract, storeCode} from "./utils.js";
import {readFile} from 'fs/promises';

import {promisify} from 'util';

import * as readline from 'node:readline';

import * as chai from 'chai';
import {MnemonicKey, MsgSend} from "@terra-money/terra.js";


const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});
const question = promisify(rl.question).bind(rl);
//------ change this value to manage the batch alloccation
const max_batch_size = 3
let new_pool_id = null;
const assert = chai.assert;
// Init and Vars
let gaming_contract_address = ""
let proxy_contract_address = "terra19zpyd046u4swqpksr3n44cej4j8pg6ah2y6dcg"
let fury_contract_address = "terra18vd8fpwxzck93qlwghaj6arh4p7c5n896xzem5"
const gamer = walletTest1.key.accAddress
// const gamer_extra_1 = walletTest3.key.accAddress
// const gamer_extra_2 = walletTest4.key.accAddress
//Deployer or wallet 1 Needs to have funds

const wallets_json = JSON.parse(
    await readFile(
        new URL('wallets.json', import.meta.url)
    )
);


const funding_wallet = walletTest1

const gaming_init = {

    "minting_contract_address": fury_contract_address, //  This should be a contract But We passed wallet so it wont raise error on addr validate
    "admin_address": walletTest1.key.accAddress,
    "platform_fee": "1",
    "transaction_fee": "1",
    "game_id": "Game001",
    "platform_fees_collector_wallet": walletTest1.key.accAddress,
    "astro_proxy_address": proxy_contract_address,
}

// Helper Methods

function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}


const deploy_contract = async function (file, init) {
    const contractId = await storeCode(walletTest1, file,)
    await sleep(sleep_time)
    const gamingInit = await instantiateContract(walletTest1, contractId, init)
    console.log(`New Contract Init Hash ${gamingInit.txhash}`)
    return gamingInit.logs[0].events[0].attributes[3].value; // Careful with order of argument
}


// Tests
let test_create_and_query_game = async function (time) {
    console.log("Uploading Gaming Contract")
    gaming_contract_address = await deploy_contract(GamingContractPath, gaming_init)
    console.log(`Gaming Address:${gaming_contract_address}`)
    await sleep(sleep_time)
    console.log("Executing Query For Contract Details")
    let query_resposne = await queryContract(gaming_contract_address, {
        game_details: {}
    })
    assert.isTrue(gaming_init['game_id'] === query_resposne['game_id'])
    assert.isTrue(1 === query_resposne['game_status'])
    console.log("Assert Success")
    await sleep(time)
}

let test_create_and_query_pool = async function (time) {
    console.log("Testing Create and Query Pool")
    console.log("Create Pool")
    let response = await executeContract(walletTest1, gaming_contract_address, {
        create_pool: {
            "pool_type": "H2H"
        }
    })
    console.log(`Pool Create TX : ${response.txhash}`)
    new_pool_id = response.logs[0].events[1].attributes[1].value
    console.log(`New Pool ID  ${new_pool_id}`)
    response = await queryContract(gaming_contract_address, {
        pool_details: {
            "pool_id": new_pool_id
        }
    })
    assert.isTrue(response['pool_id'] === new_pool_id)
    assert.isTrue(response['game_id'] === "Game001")
    assert.isTrue(response['pool_type'] === "H2H")
    assert.isTrue(response['current_teams_count'] === 0)
    assert.isTrue(response['rewards_distributed'] === false)
    console.log("Assert Success")
    await sleep(time)
}

const set_pool_headers_for_H2H_pool_type = async function (time) {
    const response = await executeContract(walletTest1, gaming_contract_address, {
        set_pool_type_params: {
            'pool_type': "H2H",
            'pool_fee': "10000000",
            'min_teams_for_pool': 1,
            'max_teams_for_pool': 2,
            'max_teams_for_gamer': 2,
            'wallet_percentages': [
                {
                    "wallet_address": "terra1uyuy363rzun7e6txdjdemqj9zua9j7wxl2lp0m",
                    "wallet_name": "rake_1",
                    "percentage": 100,
                }
            ]
        }
    })
    console.log(response)
    console.log("Assert Success")
    if (time) await sleep(time)
}

// This method will return new wallets_json for test
function get_wallets(count) {
    let wallets = []
    for (let i = 0; i < count; i++) {
        wallets.push(terraClient.wallet(null))
    }
    return wallets
}

//This will load the defined UST in all the accounts
// We do this in bathces but since the max sub per client is 5 more than 5 cocurrent request will cause this process
// exit so the safest batch limit is 4 this can be same or lower for test-net
async function load_funds(wallets, funds_per_wallet) {
    let promises_to_fulfill = []
    for (const wallet of wallets) {
        promises_to_fulfill.push(bankTransferFund(funding_wallet, wallet, 10));
        if (promises_to_fulfill.length > max_batch_size) {
            await Promise.all(promises_to_fulfill).catch(async () => {
                await sleep(10000)
                await Promise.all(promises_to_fulfill)
                promises_to_fulfill = []
            })
            promises_to_fulfill = []
        }
    }
}

// This will make all the wallets execute a given message
async function execute_from_all(wallets, contract_address, message, funds_to_send) {
    for (const wallet in wallets) {
        await executeContract(wallet, contract_address, message, funds_to_send)
    }
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

export async function wallets_to_obj(wallets) {
    let wallet_objects = []
    for (const wallet in wallets) {
        const mk = new MnemonicKey({mnemonic: wallet});
        wallet_objects.push(terraClient.wallet(mk))
        if (wallet_objects.length > 2) {
            break
        }
    }
    console.log("Wallets Ready....")
    return wallet_objects
}

//----------------------------------------- TESTING ---------------------------------------------------------------
//
// await test_create_and_query_game(sleep_time)
// await test_create_and_query_pool(sleep_time)
// await set_pool_headers_for_H2H_pool_type(sleep_time)
//
// Gaming Load Testing
// We will load and prefund all the wallets_json
// We will start with the setup for gaming
// let wallets_for_test = await wallets_to_obj(wallets_json)
// await load_funds(wallets_for_test, 100)
// // Setup A Given
//
