import {migrateContract, storeCode} from "./utils.js";
import {GamingContractPath, terraClient} from "./constants.js";
import {MnemonicKey} from "@terra-money/terra.js";
import {executeContract, queryContract} from "./utils.js";

export function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
}

let sleep_time = 31000
let some = "rookie choose awake accident brisk day shoe fashion battle cost increase wrestle oyster drill mansion prevent top leader uncle again arctic carpet mention lend"
const mkAjay = new MnemonicKey({mnemonic: some});
export const migrate = terraClient.wallet(mkAjay);

const upload_contract = async function (file) {
    const contractId = await storeCode(migrate, file,)
    console.log(`New Contract Id For Gaming ${contractId}`)
    await sleep(31000)
    await migrateContract(migrate, "terra1x7s2l8n3q5jwmaaq3wmpc3f7erspy5q7pkqasu", contractId, {})
}
upload_contract(GamingContractPath)
//
// executeContract(migrate, "terra1x7s2l8n3q5jwmaaq3wmpc3f7erspy5q7pkqasu", {
//     "game_pool_reward_distribute": {
//         "is_final_batch": true,
//         "pool_id": "9",
//         "game_id": "41222",
//         "game_winners": [
//             {
//                 "gamer_address": "terra15e8684els8teypsqu2kv2t5x6wd0zelaexznfg",
//                 "team_id": "624f689d3b763c9ffd54244b",
//                 "reward_amount": "16538558",
//                 "refund_amount": "0",
//                 "team_rank": 1,
//                 "team_points": 55,
//                 "game_id": "41222"
//             },
//             {
//                 "gamer_address": "terra1yuargn90dst8fvyjkc09s2wqfcucy9rtccnjzh",
//                 "team_id": "624f690c3b763c9ffd542539",
//                 "reward_amount": "16538558",
//                 "refund_amount": "0",
//                 "team_rank": 2,
//                 "team_points": 41,
//                 "game_id": "41222"
//             }
//         ]
//     }
// })
executeContract(migrate,"terra1x7s2l8n3q5jwmaaq3wmpc3f7erspy5q7pkqasu",{

})