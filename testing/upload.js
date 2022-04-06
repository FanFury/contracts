import {instantiateContract, storeCode} from "./utils.js";
import {GamingContractPath, terraClient} from "./constants.js";
import {MnemonicKey} from "@terra-money/terra.js";

//-------------------------------
const MEMONIC = ""
//---------------------------------

const mkSameer = new MnemonicKey({mnemonic: MEMONIC});
export const operator = terraClient.wallet(mkSameer);


const upload_contract = async function (file) {
    const contractId = await storeCode(operator, file,)
    console.log(`New Contract Id For Gaming ${contractId}`)
    return contractId
}
let contract__id = await upload_contract(GamingContractPath)
const gamingInit = await instantiateContract(operator, contract__id, {
    operator: operator.key.accAddress
})
console.log(gamingInit)
