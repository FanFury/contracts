import message from "@cosmostation/cosmosjs/src/messages/proto.js";
import fs from "fs";
import fetch from "node-fetch";
import {Cosmos} from "@cosmostation/cosmosjs";

const chainId = "juno"
const lcdUrl = "http://localhost:1317"
// Copy Memonic from the Terminal in which the Juno Node contrainer was upped
export const mnemonic = "soup travel dinosaur remember rally leader prosper live during tent change lend hollow acquire expand maze dove walnut lumber mammal song raw decline draft"
export const cosmos = new Cosmos(lcdUrl, chainId);
cosmos.setBech32MainPrefix("juno")
console.log(cosmos.bech32MainPrefix)

export class Wallet {
    wallet_address;
    publicKey;
    privateKey;

    constructor(memonic) {
        this.privateKey = cosmos.getECPairPriv(memonic);
        this.publicKey = cosmos.getPubKeyAny(this.privateKey);
        this.wallet_address = cosmos.getAddress(memonic);
        this.url = cosmos.url
        this.feeValue = new message.cosmos.tx.v1beta1.Fee({
            amount: [{denom: "ujunox", amount: String(20000)}],
            gas_limit: 100000000
        });
    }

    async sign_and_broadcast(messages) {
        return cosmos.getAccounts(this.wallet_address).then(async data => {
            let signerInfo = new message.cosmos.tx.v1beta1.SignerInfo({
                public_key: this.publicKey,
                mode_info: {single: {mode: message.cosmos.tx.signing.v1beta1.SignMode.SIGN_MODE_DIRECT}},
                sequence: data.account.sequence
            });
            const txBody = new message.cosmos.tx.v1beta1.TxBody({messages: messages, memo: ""});
            const authInfo = new message.cosmos.tx.v1beta1.AuthInfo({signer_infos: [signerInfo], fee: this.feeValue});
            const signedTxBytes = cosmos.sign(txBody, authInfo, data.account.account_number, this.privateKey);
            return cosmos.broadcast(signedTxBytes, "BROADCAST_MODE_BLOCK")
        })
    }

    send_funds(to_address, coins) {
        const msgSend = new message.cosmos.bank.v1beta1.MsgSend({
            from_address: this.wallet_address,
            to_address: to_address,
            amount: [coins]
        });

        return this.sign_and_broadcast([{
            type_url: "/cosmos.bank.v1beta1.MsgSend",
            value: message.cosmos.bank.v1beta1.MsgSend.encode(msgSend).finish()
        }])
    }

    async execute_contract(msg, contractAddress, coins) {
        let msg_list = []
        if (Array.isArray(msg)) {
            msg.forEach((msg) => {
                msg_list.push(this.get_execute(msg, contractAddress, coins))
            })

        } else {
            msg_list = [
                this.get_execute(msg, contractAddress)
            ]
        }
        let response = await this.sign_and_broadcast(msg_list)
        console.log(response)
        return response
    }

    get_execute(msg, contract, coins) {
        let transferBytes = new Buffer.from(JSON.stringify(msg));
        const msgExecuteContract = new message.cosmwasm.wasm.v1.MsgExecuteContract({
            sender: this.wallet_address,
            contract: contract,
            msg: transferBytes,
            funds: coins
        });
        return new message.google.protobuf.Any({
            type_url: "/cosmwasm.wasm.v1.MsgExecuteContract",
            value: message.cosmwasm.wasm.v1.MsgExecuteContract.encode(msgExecuteContract).finish()
        })
    }

    query(address, query) {
        cosmos.wasmQuery(
            address,
            JSON.stringify(query)
        ).then(json => {
            return json
        })
    }

    async upload(file) {
        const code = fs.readFileSync(file).toString("base64");
        const msgStoreCode = new message.cosmwasm.wasm.v1.MsgStoreCode({
            sender: this.wallet_address,
            wasm_byte_code: code,
        });
        let response = await this.sign_and_broadcast([{
            type_url: "/cosmwasm.wasm.v1.MsgStoreCode",
            value: message.cosmwasm.wasm.v1.MsgStoreCode.encode(msgStoreCode).finish()
        }])
        console.log(response.tx_response.raw_log)
        let j = JSON.parse(response.tx_response.raw_log)
        return parseInt(j[0].events[1].attributes[0].value)
    }

    async init(code_id, contract_init) {
        let transferBytes = new Buffer.from(JSON.stringify(contract_init));
        const msgInit = new message.cosmwasm.wasm.v1.MsgInstantiateContract({
            sender: this.wallet_address,
            admin: this.wallet_address,
            code_id: parseInt(code_id),
            msg: transferBytes,
            label: "some",
            initFunds: []
        });
        let response = await this.sign_and_broadcast([{
            type_url: "/cosmwasm.wasm.v1.MsgInstantiateContract",
            value: message.cosmwasm.wasm.v1.MsgInstantiateContract.encode(msgInit).finish()
        }])
        console.log("Events")
        console.log(response.tx_response.events)
        for (let i = 0; i < response.tx_response.events.length; i++) {
            console.log(response.tx_response.events[i])
            let attr = response.tx_response.events[i].attributes
            for (let j = 0; j < attr.length; j++) {
                console.log(attr[j])
            }
        }
        let address = Buffer.from(response.tx_response.events[response.tx_response.events.length - 1].attributes[0].value, "base64").toString()
        if (address.includes("juno")) {
            return address
        }
        throw new Error("Error Instantiating the contract, please check the init message and try again...")
    }


    sleep(time) {
        return new Promise((resolve) => setTimeout(resolve, time));
    }

    queryBankUusd(address) {
        let api = "/cosmos/bank/1beta1/balances/";
        return fetch(this.url + api + address).then(response => response.json())
    }


}

let wallet = new Wallet(mnemonic)
// let response = await wallet.upload("../../artifacts/vest_n_distribute.wasm")
// console.log(response)
// wallet.send_funds("juno1gcxq5hzxgwf23paxld5c9z0derc9ac4m5g63xa", {denom: "ujunox", amount: String(100)})