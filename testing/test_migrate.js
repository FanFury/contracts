import {instantiateContract, migrateContract, storeCode} from "./utils.js";
import {ClubStakingContractPath, GamingContractPath, walletTest1} from "./constants.js";


const deploy_contract = async function (file, user, init) {
    const contractId = await storeCode(user, file,)
    const gamingInit = await instantiateContract(user, contractId, init)
    console.log(`New Contract Init Hash ${gamingInit.txhash}`)
    return gamingInit.logs[0].events[0].attributes[3].value; // Careful with order of argument
}

const upload_init_and_migrate = async function (file, user, init) {
    let address = await deploy_contract(file, user, init)
    let contract_id = await storeCode(user, file,) //  Uploading the new contract
    let r = await migrateContract(user, address, contract_id, {})
    console.log(r)
    console.log("Success")
}

//------------------------------

await upload_init_and_migrate(GamingContractPath, walletTest1, {
    "minting_contract_address": walletTest1.key.accAddress, //  This should be a contract But We passed wallet so it wont raise error on addr validate
    "admin_address": walletTest1.key.accAddress,
    "platform_fee": "1",
    "transaction_fee": "1",
    "game_id": "Game001",
    "platform_fees_collector_wallet": walletTest1.key.accAddress,
    "astro_proxy_address": walletTest1.key.accAddress,
})

await upload_init_and_migrate(ClubStakingContractPath, walletTest1, {
    admin_address: walletTest1.key.accAddress,
    minting_contract_address: walletTest1.key.accAddress,
    astro_proxy_address: walletTest1.key.accAddress,
    platform_fees_collector_wallet: walletTest1.key.accAddress,
    club_fee_collector_wallet: walletTest1.key.accAddress,
    club_reward_next_timestamp: "1640447808000000000",
    reward_periodicity: 300,
    club_price: "100000",
    bonding_duration: 120,
    owner_release_locking_duration: 120,
    platform_fees: "100",
    transaction_fees: "30",
    control_fees: "50",
})