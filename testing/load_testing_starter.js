import {MnemonicKey} from "@terra-money/terra.js";
import {terraClient, walletTest1} from "./constants.js";
import {executeContract} from "./utils";
/*
This script is meant to be used as a boiler-plate for setting up load testing .
Here we can perform the initial setup of the contract workflow and then send
the main endpoint for bulk_testing to
*/
//This is the wallet that will fund all the new wallets
const funding_wallet = walletTest1

const wallets_json = JSON.parse(
    await readFile(
        new URL('wallets.json', import.meta.url)
    )
);

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

// Inital Setup
let wallets_for_test = await wallets_to_obj(wallets_json)
await load_funds(wallets_for_test, 100)
// Run the contract setup and deployment
//-----------------------
//execute_from_all  -  on the method that needs to be executed by multiple users to generate and test the load