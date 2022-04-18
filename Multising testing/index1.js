const { calculateFee, GasPrice, isMsgSubmitProposalEncodeObject } = require("@cosmjs/stargate");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { SigningCosmWasmClient, CosmWasmClient } = require("@cosmjs/cosmwasm-stargate");
const _ = require("fs");
const btoa = require("btoa");
const atob = require("atob")

const rpcEndpoint = "https://rpc.cliffnet.cosmwasm.com:443";

// Example user from scripts/wasmd/README.md
const sender = {
    mnemonic: "approve year inmate lion practice cat kite sweet amazing brain earn topple atom upper alpha control lunar glance spell old place strategy fly wrap",
    address: "wasm1empqdlq6rc2jg6eg7a6lz2yw46yff4yqspjtmy",
};

const vote1 = {
    mnemonic: "tilt panda season try carpet kick together idea trophy toe session attitude now rescue result eternal update valley betray mirror hair ketchup traffic sustain",
    address: "wasm1tpv643x4rkggppmvquf5j5l0cjsnggr443r9x9",
};

const vote2 = {
    mnemonic: "vivid idea danger similar spot castle flip lonely help simple minute blossom silly practice game another hub quiz barely gloom place goddess fetch diet",
    address: "wasm18evhs6mpudcqcduyprsrr6gqughtgyyx5sja3c",
};

const vote3 = {
    mnemonic: "pact hour vivid bounce rough exile armed whale hero still rice stereo baby bomb rather poverty initial trust modify brush battle sun behave opera",
    address: "wasm1cwkve89p5yzemkftjgkugr98xzpyakd2seus20",
}

const vote4 = {
    mnemonic: "divide cement minute silly mask ring fresh captain roast menu degree pill few permit obtain long emotion jar pen such stable any obey repair",
    address: "wasm1dqrt7sm8wlawjcf9n8er5xlchry8lulfkl9nn4",
}


//************************************************* */
//uploading//
//**************************************************//
async function uploadMain() {
    const escrowWasmPath = "./multisig.wasm";
    const gasPrice = GasPrice.fromString("0.05upebble");

    // Upload contract
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const wasm = await _.readFileSync(escrowWasmPath);
    const uploadFee = await calculateFee(3_000_0000, gasPrice);
    const uploadVault = await sender_client.upload(sender.address, wasm, uploadFee, "Upload hackatom contract");
    console.log("Upload succeeded. Receipt:", uploadVault);
    return uploadVault.codeId;
}

async function uploadCw3() {
    const escrowWasmPath = "./cw3_flex_multisig.wasm";
    const gasPrice = GasPrice.fromString("0.05upebble");

    // Upload contract
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const wasm = await _.readFileSync(escrowWasmPath);
    const uploadFee = await calculateFee(3_000_0000, gasPrice);
    const uploadVault = await sender_client.upload(sender.address, wasm, uploadFee, "Upload hackatom contract");
    console.log("Upload succeeded. Receipt:", uploadVault);
    return uploadVault.codeId;
}

async function uploadCw4() {
    const escrowWasmPath = "./cw4_group.wasm";
    const gasPrice = GasPrice.fromString("0.05upebble");

    // Upload contract
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const wasm = await _.readFileSync(escrowWasmPath);
    const uploadFee = await calculateFee(3_000_0000, gasPrice);
    const uploadVault = await sender_client.upload(sender.address, wasm, uploadFee, "Upload hackatom contract");
    console.log("Upload succeeded. Receipt:", uploadVault);
    return uploadVault.codeId;
}

async function uploadCw20() {
    const escrowWasmPath = "./cw20_base.wasm";
    const gasPrice = GasPrice.fromString("0.05upebble");

    // Upload contract
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const wasm = await _.readFileSync(escrowWasmPath);
    const uploadFee = await calculateFee(3_000_0000, gasPrice);
    const uploadVault = await sender_client.upload(sender.address, wasm, uploadFee, "Upload hackatom contract");
    console.log("Upload succeeded. Receipt:", uploadVault);
    return uploadVault.codeId;
}

//***************************************************************************** */
//instantiate
//***************************************************************************** */


async function instantiateMain(codeId) {
    // // Instantiate
    const gasPrice = GasPrice.fromString("0.05upebble");
    const instantiateFee = calculateFee(500_000, gasPrice);
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const initMsg = {

    };

    const { contractAddress } = await sender_client.instantiate(
        sender.address,
        codeId,
        initMsg,
        "STAR",
        instantiateFee,
        { memo: `Create a hackatom instance` },
    );
    console.info(`Contract instantiated at: `, contractAddress);

}

async function instantiateCw20(codeId, address) {
    // // Instantiate
    const gasPrice = GasPrice.fromString("0.05upebble");
    const instantiateFee = calculateFee(500_000, gasPrice);
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const initMsg = {
        name: "Golden Stars",
        symbol: "STAR",
        decimals: 2,
        // list of all validator self-delegate addresses - 100 STARs each!
        initial_balances: [],
        mint: {
            minter: address,
        },
    };

    const { contractAddress } = await sender_client.instantiate(
        sender.address,
        codeId,
        initMsg,
        "STAR",
        instantiateFee,
        { memo: `Create a hackatom instance` },
    );
    console.info(`Contract instantiated at: `, contractAddress);

}

async function instantiateCw4(codeId, address) {
    // // Instantiate
    const gasPrice = GasPrice.fromString("0.05upebble");
    const instantiateFee = calculateFee(500_000, gasPrice);
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const initMsg = {
        admin: sender.address,
        members: [{ addr: sender.address, weight: 1 },
        { addr: vote1.address, weight: 1 },
        { addr: vote2.address, weight: 1 },
        { addr: vote3.address, weight: 1 }]
    };

    const { contractAddress } = await sender_client.instantiate(
        sender.address,
        codeId,
        initMsg,
        "STAR",
        instantiateFee,
        { memo: `Create a hackatom instance` },
    );
    console.info(`Contract instantiated at: `, contractAddress);

}


async function instantiateCw3(codeId, address) {
    // // Instantiate
    const gasPrice = GasPrice.fromString("0.05upebble");
    const instantiateFee = calculateFee(500_000, gasPrice);
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const initMsg = {
        group_addr: address,
        threshold: {
            absolute_percentage: { percentage: "0.67" }
        },
        max_voting_period: { time: 600 }
    };

    const { contractAddress } = await sender_client.instantiate(
        sender.address,
        codeId,
        initMsg,
        "STAR",
        instantiateFee,
        { memo: `Create a hackatom instance` },
    );
    console.info(`Contract instantiated at: `, contractAddress);

}

async function addHook(cw4address, cw3Address) {
    const gasPrice = GasPrice.fromString("0.05upebble");
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(sender.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const executeFee = calculateFee(300_000, gasPrice);
    const msg =
    {
        add_hook: {
            addr: cw3Address
        }
    }
    const buy_token = await sender_client.execute(
        sender.address,
        cw4address,
        msg,
        executeFee,
        // "",
        // [{ denom: "ujunox", amount: "10000" }]
    );
    console.log("Adding hook", buy_token)
}


async function proposeTransaction(contractAddress, cw20Address, address, amount) {
    const gasPrice = GasPrice.fromString("0.05upebble");
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(vote1.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const executeFee = calculateFee(300_000, gasPrice);

    const mintMsg = {
        transfer_from: {
            owner: vote1.address,
            recipient: address,
            amount: amount
        }
    }

    const subMsg = [
        {
            wasm: {
                execute: {
                    contract_addr: cw20Address,
                    msg: btoa(JSON.stringify(mintMsg)),
                    funds: []
                }
            }
        }
    ];

    const msg = {
        propose: {
            title: "Mint",
            description: "mint",
            msgs: subMsg,
        }
    }

    // console.log(msg)
    const propose = await sender_client.execute(
        vote1.address,
        contractAddress,
        msg,
        executeFee,
        // "",
        // [{ denom: "ujunox", amount: "10000" }]
    );
    console.log("Propose", propose)
}

async function getListProposal(contractAddress) {

    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(vote1.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const msg = {
        list_proposals: {}
    }
    const create_result = await sender_client.queryContractSmart(
        //sender.address,
        contractAddress,
        msg
    );
    console.log(create_result)
}

async function getBalance(contractAddress, address) {

    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(vote1.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const msg = {
        balance: {
            address: address
        }
    }
    const create_result = await sender_client.queryContractSmart(
        //sender.address,
        contractAddress,
        msg
    );
    console.log(create_result)
}


async function getProposalInfo(contractAddress, id) {
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(vote1.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const msg = {
        proposal: {
            proposal_id: id
        }
    }
    const create_result = await sender_client.queryContractSmart(
        //sender.address,
        contractAddress,
        msg
    );
    console.log(create_result)
}

async function executeVote(cw3Address, id, voter) {
    const gasPrice = GasPrice.fromString("0.05upebble");
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(voter.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const executeFee = calculateFee(300_000, gasPrice);
    const msg =
    {
        vote: {
            proposal_id: id,
            vote: "yes"
        }
    }
    const vote = await sender_client.execute(
        voter.address,
        cw3Address,
        msg,
        executeFee,
        // "",
        // [{ denom: "ujunox", amount: "10000" }]
    );
    console.log("Voting", vote)
}



async function executePropose(cw3Address, cw20Address, id) {

    const gasPrice = GasPrice.fromString("0.05upebble");
    const sender_wallet = await DirectSecp256k1HdWallet.fromMnemonic(vote1.mnemonic, { prefix: "wasm" });
    const sender_client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, sender_wallet);
    const executeFee = calculateFee(300_000, gasPrice);

    const msg = {
        proposal: {
            proposal_id: id
        }
    }
    const propose_info = await sender_client.queryContractSmart(
        //sender.address,
        cw3Address,
        msg
    );



    const amount = JSON.parse(atob(propose_info.msgs[0].wasm.execute.msg)).transfer_from.amount;

    if (propose_info.status == "passed") {
        const allowance_msg = {
            increase_allowance: {
                spender: cw3Address,
                amount: String(amount)
            }
        }

        const allowance = await sender_client.execute(
            vote1.address,
            cw20Address,
            allowance_msg,
            executeFee,
            // "",
            // [{ denom: "ujunox", amount: "10000" }]
        );
        console.log("Increase Allownace", allowance)

        const execute_msg =
        {
            execute: {
                proposal_id: id,
            }
        }
        const execute = await sender_client.execute(
            vote1.address,
            cw3Address,
            execute_msg,
            executeFee,
            // "",
            // [{ denom: "ujunox", amount: "10000" }]
        );
        console.log("Executing", execute)
    }
    else {
        console.log("Status is wrong")
    }
}



async function main() {
    //codeMainId = uploadMain();
    //codeCW3 = uploadCw3();//1001
    // codeCW4 = await uploadCw4();//1002
    //codeCw20 = await uploadCw20();//1003
    const codeMainId = 1052;
    const codeCW3 = 1054;
    const codeCW4 = 1055;
    const codeCw20 = 1056;
    const mainAddress = "wasm100ucwezg08nkwuww5d3ehx5h8mqu00nr0pmukpkstmapuques9dspqmy5c";
    const cw20Address = "wasm1pdknh3e4x6d8v29nj4qyzqtcgz2sk3awhnr2se9d3qqdd9g95quqn3xah9";
    const cw4Address = "wasm1l3k402qcsg7375zc5x7slujd7nushqaehrh5wyvr9xu36k5atw3q94408s";
    const cw3Address = "wasm1asnk3q6009yrvydfqsn240xwtzwyjmu5xt8s097h2zamz2s2hn5q4q7x5v";
    // const mainaddress = await instantiateMain(codeMainId)
    // const cw20Address = await instantiateCw20(codeCw20, cw3Address);
    // const cw4Address = await instantiateCw4(codeCW4)
    // const cw3Address = await instantiateCw3(codeCW3, cw4Address);
    //await addHook(cw4Address, cw3Address);
    await proposeTransaction(cw3Address, cw20Address, vote2.address, "100");
    const id = 10;
    await getListProposal(cw3Address);
    await executeVote(cw3Address, id, vote3)
    await executeVote(cw3Address, id, vote2)
    await executePropose(cw3Address, cw20Address, id)
    await getBalance(cw20Address, vote2.address)
}

main();