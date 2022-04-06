import {storeCode} from "./utils.js";
import {GamingContractPath, walletTest1} from "./constants.js";

const upload_contract = async function (file) {
    const contractId = await storeCode(walletTest1, file,)
    console.log(`New Contract Id For Gaming ${contractId}`)
}
await upload_contract(GamingContractPath)