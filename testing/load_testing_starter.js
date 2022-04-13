import {MnemonicKey, MsgSend} from "@terra-money/terra.js";
import {terraClient, walletTest1} from "./constants.js";
import {executeContract} from "./utils";
import {readFile} from 'fs/promises';
import {sleep} from "./migrate.js";

/*
This script is meant to be used as a boiler-plate for setting up load testing .
Here we can perform the initial setup of the contract workflow and then send
the main endpoint for bulk_testing to execute_from_all
*/
//This is the wallet that will fund all the new wallets
const funding_wallet = walletTest1

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
``
const wallets_json = JSON.parse(
    await readFile(
        new URL('wallets.json', import.meta.url)
    )
);

async function load_funds(wallets, funds_per_wallet) {
    let promises_to_fulfill = []
    for (const wallet of wallets) {
        promises_to_fulfill.push(bankTransferFund(funding_wallet, wallet, 10));
        if (promises_to_fulfill.length > 3) {
            await Promise.all(promises_to_fulfill).catch(async () => {
                await sleep(10000)
                await Promise.all(promises_to_fulfill)
                promises_to_fulfill = []
            })
            promises_to_fulfill = []
        }
    }
}

async function wallets_to_obj(wallets) {
    let wallet_objects = []
    for (const wallet in wallets) {
        const mk = new MnemonicKey({mnemonic: wallet});
        wallet_objects.push(terraClient.wallet(mk))
    }
    console.log("Wallets Ready....")
    return wallet_objects
}

async function execute_from_all(wallets, contract_address, message, funds_to_send) {
    for (const wallet in wallets) {
        await executeContract(wallet, contract_address, message, funds_to_send)
    }
}
//----------------------------------------- TESTING ---------------------------------------------------------------
//
// Inital Setup
let wallets_for_test = await wallets_to_obj(wallets_json)
await load_funds(wallets_for_test, 100)
// Run the contract setup and deployment
//-----------------------
//execute_from_all  -  on the method that needs to be executed by multiple users to generate and test the load