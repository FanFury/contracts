import {instantiateContract, sleep, storeCode} from "./utils.js";
import {PrismContractPath, terraClient} from "./constants.js";
import {MnemonicKey} from "@terra-money/terra.js";

const sleep_time = 0
//---------------------USER MEMONIC HERE---------------------
const memonic = ""
//------------------------------------------------------------
const mkSameer = new MnemonicKey({mnemonic: memonic});
let deployer = terraClient.wallet(mkSameer);


let prism_init = {
  "owner": "OWNER_ADDRESS",
  "denom": "uusd"
}

console.log("Uploading Contract")
const contractId = await storeCode(deployer, PrismContractPath)
await sleep(sleep_time)
console.log(`New Contract ID ${contractId}`)
let response = await instantiateContract(deployer, contractId, prism_init)
console.log(`Response From Prism Init ${response.txhash}`)
