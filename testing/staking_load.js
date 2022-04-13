import {ClubStakingContractPath, sleep_time, terraClient, walletTest1, nitin_wallet} from './constants.js';
import {getGasUsed, instantiateContract, executeContract, queryContract, readArtifact, storeCode, writeArtifact } from './utils.js';
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
let club_staking_contract_address = ""
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
    club_staking_contract_address = await deploy_contract(GamingContractPath, gaming_init)
    console.log(`Gaming Address:${club_staking_contract_address}`)
    await sleep(sleep_time)
    console.log("Executing Query For Contract Details")
    let query_resposne = await queryContract(club_staking_contract_address, {
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
    let response = await executeContract(walletTest1, club_staking_contract_address, {
        create_pool: {
            "pool_type": "H2H"
        }
    })
    console.log(`Pool Create TX : ${response.txhash}`)
    new_pool_id = response.logs[0].events[1].attributes[1].value
    console.log(`New Pool ID  ${new_pool_id}`)
    response = await queryContract(club_staking_contract_address, {
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
    const response = await executeContract(walletTest1, club_staking_contract_address, {
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
    for (const wallet in wallets) {
        promises_to_fulfill.push(bankTransferFund(funding_wallet, wallet, funds_per_wallet));
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


async function wallets_to_obj(wallets) {
    let wallet_objects = []
	let wallet_obj;
    for (const wallet in wallets) {
        const mk = new MnemonicKey({mnemonic: wallet});
		wallet_obj = terraClient.wallet(mk);
        wallet_objects.push(wallet_obj);
    }
    console.log("Wallets Ready....");
	return wallet_objects;
}

async function transferFuryToWallet(wallet_from, wallet_to, fury_amount) {
		let transferFuryMsg = {
			transfer: {
				recipient: wallet_to.key.accAddress,
				amount: fury_amount.toString()
			}
		};
		console.log(`transferFuryMsg = ${JSON.stringify(transferFuryMsg)}`);
		let response = await executeContract(wallet_from, fury_contract_address, transferFuryMsg);
		console.log(`transferFuryMsg Response - ${response['txhash']}`);
}

async function fundAllMainWallets(deploymentDetails) {
    await transferFuryToNitin(deploymentDetails);
}

async function transferFuryToNitin(deploymentDetails) {
    let transferFuryToNitinMsg = {
        transfer: {
            recipient: nitin_wallet.key.accAddress,
            amount: "50000000"
        }
    };
    console.log(`transferFuryToNitinMsg = ${JSON.stringify(transferFuryToNitinMsg)}`);
    let response = await executeContract(funding_wallet, fury_contract_address, transferFuryToNitinMsg);
    console.log(`transferFuryToNitinMsg Response - ${response['txhash']}`);
}


async function uploadClubStaking(deploymentDetails) {
    if (!deploymentDetails.clubStakingId) {
        console.log("Uploading Club Staking...");
        let contractId = await storeCode(mint_wallet, ClubStakingContractPath); // Getting the contract id from local terra
        console.log(`Club Staking Contract ID: ${contractId}`);
        deploymentDetails.clubStakingId = contractId;
        writeArtifact(deploymentDetails, terraClient.chainID);
    }
}


async function main() {
	console.log("funding all main wallets");
    terraClient.chainID = "localterra";
    let deploymentDetails = readArtifact(terraClient.chainID);
	await fundAllMainWallets(deploymentDetails);
	console.log("now funding all json wallets");
	// We will load and prefund all the wallets_json
	let wallets_for_test = await wallets_to_obj(wallets_json);
	wallets_for_test.forEach(wallet_to => {
		console.log(`wallet = ${wallet_to.key.accAddress}`);
		bankTransferFund(funding_wallet, wallet_to, 1000000);
		transferFuryToWallet(funding_wallet, wallet_to, 1000000);
	})
	console.log("end of execution");
	process.exit();
}


async function uploadClubStaking(deploymentDetails) {
    if (!deploymentDetails.clubStakingId) {
        console.log("Uploading Club Staking...");
        let contractId = await storeCode(mint_wallet, ClubStakingContractPath); // Getting the contract id from local terra
        console.log(`Club Staking Contract ID: ${contractId}`);
        deploymentDetails.clubStakingId = contractId;
        writeArtifact(deploymentDetails, terraClient.chainID);
    }
}

async function instantiateClubStaking(deploymentDetails) {
    if (!deploymentDetails.clubStakingAddress) {
        console.log("Instantiating Club Staking...");
        /*
        Club Price in this contract is 100000 (0.1 Fury) -  "club_price": "100000"
        Withdraw from a Club will mature after 2 minutes 120 seconds -  "bonding_duration": 120
        Also a repeat calculate_and_distribute_reward()
            if called within 5 minutes shall fail - "reward_periodicity": 300
        */
        let clubStakingInitMessage = {
            admin_address: deploymentDetails.adminWallet,
            minting_contract_address: deploymentDetails.furyContractAddress,
            astro_proxy_address: deploymentDetails.astroProxyContractAddress,
            platform_fees_collector_wallet: deploymentDetails.adminWallet,
            club_fee_collector_wallet: deploymentDetails.teamWallet,
            club_reward_next_timestamp: "1640447808000000000",
            reward_periodicity: 300,
            club_price: "100000",
            bonding_duration: 120,
            platform_fees: "100",
            transaction_fees: "30",
            control_fees: "50",
            max_bonding_limit_per_user: 100,
        }
        console.log(JSON.stringify(clubStakingInitMessage, null, 2));
        let result = await instantiateContract(mint_wallet, deploymentDetails.clubStakingId, clubStakingInitMessage);
        let contractAddresses = result.logs[0].events[0].attributes.filter(element => element.key == 'contract_address').map(x => x.value);
        deploymentDetails.clubStakingAddress = contractAddresses.shift();
        console.log(`Club Staking Contract Address: ${deploymentDetails.clubStakingAddress}`);
        writeArtifact(deploymentDetails, terraClient.chainID);
    }
}

async function performOperationsOnClubStaking(deploymentDetails) {
    await queryAllClubOwnerships(deploymentDetails);
    console.log("Balances of buyer before buy club");
    await buyAClub(deploymentDetails);
    await stakeOnAClub(deploymentDetails);
	await queryAllClubStakes(deploymentDetails);
	await distributeRewards(deploymentDetails);
}

async function queryAllClubOwnerships(deploymentDetails) {
    //Fetch club ownership details for all clubs
    let coResponse = await queryContract(deploymentDetails.clubStakingAddress, {
        all_club_ownership_details: {}
    });
    console.log("All clubs ownership = " + JSON.stringify(coResponse));

}

async function queryAllClubStakes(deploymentDetails) {
    //Fetch club Stakes details for all clubs
    let csResponse = await queryContract(deploymentDetails.clubStakingAddress, {
        club_staking_details: {
            club_name: "ClubB"
        }
    });
    console.log("All clubs stakes = " + JSON.stringify(csResponse));

}

async function buyAClub(deploymentDetails) {
    if (!deploymentDetails.clubBought) {
        //let Nitin buy a club
        // first increase allowance for club staking contract on nitin wallet to let it move fury
        let increaseAllowanceMsg = {
            increase_allowance: {
                spender: deploymentDetails.clubStakingAddress,
                amount: "100000"
            }
        };
        let incrAllowResp = await executeContract(nitin_wallet, deploymentDetails.furyContractAddress, increaseAllowanceMsg);
        console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);

        let bacRequest = {
            buy_a_club: {
                buyer: nitin_wallet.key.accAddress,
                club_name: "ClubB"
            }
        };
        let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(bacRequest)).toString('base64') } });
        console.log(`platformFees = ${JSON.stringify(platformFees)}`);
        let bacResponse = await executeContract(nitin_wallet, deploymentDetails.clubStakingAddress, bacRequest, { 'uusd': Number(platformFees) });
        console.log("Buy a club transaction hash = " + bacResponse['txhash']);
        deploymentDetails.clubBought = true;
		writeArtifact(deploymentDetails, terraClient.chainID);
    }
}

async function stakeOnAClub(deploymentDetails) {
    //let Sameer stakeOn a club
    // first increase allowance for club staking contract on Sameer wallet to let it move fury
    let increaseAllowanceMsg = {
        increase_allowance: {
            spender: deploymentDetails.clubStakingAddress,
            amount: "100000"
        }
    };
    let incrAllowResp = await executeContract(sameer_wallet, deploymentDetails.furyContractAddress, increaseAllowanceMsg);
    console.log(`Increase allowance response hash = ${incrAllowResp['txhash']}`);

    let soacRequest = {
        stake_on_a_club: {
            staker: sameer_wallet.key.accAddress,
            club_name: "ClubB",
            amount: "100000"
        }
    };
    let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(soacRequest)).toString('base64') } });
    console.log(`platformFees = ${JSON.stringify(platformFees)}`);
    let soacResponse = await executeContract(sameer_wallet, deploymentDetails.clubStakingAddress, soacRequest, { 'uusd': Number(platformFees) });
    console.log("Stake on a club transaction hash = " + soacResponse['txhash']);
}

async function withdrawStakeFromAClub(deploymentDetails) {
    let wsfacRequest = {
        stake_withdraw_from_a_club: {
            staker: sameer_wallet.key.accAddress,
            club_name: "ClubB",
            amount: "10000",
            immediate_withdrawal: false
        }
    };
    let platformFees = await queryContract(deploymentDetails.clubStakingAddress, { query_platform_fees: { msg: Buffer.from(JSON.stringify(wsfacRequest)).toString('base64') } });
    console.log(`platformFees = ${JSON.stringify(platformFees)}`);

    let wsfacResponse = await executeContract(sameer_wallet, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
    console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);

    console.log("Waiting for 30sec to try early Withdraw - would fail");
    //ADD DELAY small to check failure of quick withdraw - 30sec
    await new Promise(resolve => setTimeout(resolve, 30000));

    wsfacRequest = {
        stake_withdraw_from_a_club: {
            staker: sameer_wallet.key.accAddress,
            club_name: "ClubB",
            amount: "10000",
            immediate_withdrawal: true
        }
    };
    try {
        wsfacResponse = await executeContract(sameer_wallet, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
        console.log("Not expected to reach here");
        console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);
    } catch (error) {
        console.log("Failure as expected");
        console.log("Waiting for 100sec to try Withdraw after bonding period 2min- should pass");
        //ADD DELAY to reach beyond the bonding duration - 2min
        await new Promise(resolve => setTimeout(resolve, 100000));

        wsfacResponse = await executeContract(sameer_wallet, deploymentDetails.clubStakingAddress, wsfacRequest, { 'uusd': Number(platformFees) });
        console.log("Withdraw Stake on a club transaction hash = " + wsfacResponse['txhash']);
    } finally {
        console.log("Withdraw Complete");
    }
}

async function distributeRewards(deploymentDetails) {
    let iraRequest = {
        increase_reward_amount: {
            reward_from: mint_wallet.key.accAddress
        }
    };
	let msgString = Buffer.from(JSON.stringify(iraRequest)).toString('base64');

	let viaMsg = {
		send : {
			contract: deploymentDetails.clubStakingAddress,
			amount: "1000",
			msg: msgString
		}
	};

    let iraResponse = await executeContract(mint_wallet, deploymentDetails.furyContractAddress, viaMsg);

    //ADD DELAY small to check failure of quick withdraw - 30sec
    await new Promise(resolve => setTimeout(resolve, 30000));

    let cadrRequest = {
        calculate_and_distribute_rewards: {
        }
    };

	let cadrResponse = await executeContract(mint_wallet, deploymentDetails.clubStakingAddress, cadrRequest);
	console.log("distribute reward transaction hash = " + cadrResponse['txhash']);
}

async function queryBalances(deploymentDetails, accAddress) {
    let bankBalances = await terraClient.bank.balance(accAddress);
    console.log(JSON.stringify(bankBalances));
    let furyBalance = await queryContract(deploymentDetails.furyContractAddress, { balance: { address: accAddress } });
    console.log(JSON.stringify(furyBalance));
}
main();
