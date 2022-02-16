import {LocalTerra} from "@terra-money/terra.js";
import {get_server_epoch_seconds} from "./utils.js";
import {MnemonicKey} from '@terra-money/terra.js';

// Contracts
export const MintingContractPath = "../artifacts/cw20_base.wasm"
export const PairContractPath = "../artifacts/astroport_pair.wasm"
export const StakingContractPath = "../artifacts/astroport_staking.wasm"
export const WhitelistContractPath = "../artifacts/astroport_whitelist.wasm"
export const FactoryContractPath = "../artifacts/astroport_factory.wasm"
export const ProxyContractPath = "../artifacts/astroport_proxy.wasm"

export const terraClient = new LocalTerra();

// Accounts
export const deployer = terraClient.wallets.test1; // used as operator on all contracts
// These can be the client wallets to interact
export const walletTest1 = terraClient.wallets.test1;
export const walletTest2 = terraClient.wallets.test2;
export const walletTest3 = terraClient.wallets.test3;
export const walletTest4 = terraClient.wallets.test4;
export const walletTest10 = terraClient.wallets.test10;
// export const mint_wallet = "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k";
// export const gamifiedairdrop = "terra1m46vy0jk9wck6r9mg2n8jnxw0y4g4xgl3csh9h";
// export const privatecategory = "terra1k20rlfj3ea47zjr2sp672qqscck5k5mf3uersq";
// export const marketing = "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4";
// export const advisory = "terra19rgzfvlvq0f82zyy4k7whrur8x9wnpfcj5j9g7";
// export const sameerkey = "terra12g4sj6euv68kgx40k7mxu5xlm5sfat806umek7";
const mk1 = new MnemonicKey({mnemonic: "awesome festival volume rifle diagram suffer rhythm knock unlock reveal marine transfer lumber faint walnut love hover beach amazing robust oppose moon west will",});
export const mint_wallet = terraClient.wallet(mk1);

const mk2 = new MnemonicKey({mnemonic: "kiwi habit donor choice control fruit fame hamster trip aerobic juice lens lawn popular fossil taste venture furnace october income advice window opera helmet",});
export const treasury_wallet = terraClient.wallet(mk2);

const mk3 = new MnemonicKey({mnemonic: "job dilemma fold hurry solar strong solar priority lawsuit pass demise senior purpose useless outdoor jaguar identify enhance dirt vehicle fun nasty dragon still",});
export const liquidity_wallet = terraClient.wallet(mk3);

const mk4 = new MnemonicKey({mnemonic: "snap merit day trash key reopen stamp normal diagram vacant economy donate winner sister aerobic artist cheese bright palace athlete mind snack crawl bridge",});
export const marketing_wallet = terraClient.wallet(mk4);

export const swapinitMessage = {
    pair_code_id: 321,
    token_code_id: 123
}

export const mintInitMessage = {
    name: "Fury",
    symbol: "FURY",
    decimals: 6,
    initial_balances: [
        {address: "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k",amount: "410000000000000"},
        {address: "terra1m46vy0jk9wck6r9mg2n8jnxw0y4g4xgl3csh9h",amount: "0"},
        {address: "terra1k20rlfj3ea47zjr2sp672qqscck5k5mf3uersq",amount: "0"},
        {address: "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4",amount: "0"},
        {address: "terra19rgzfvlvq0f82zyy4k7whrur8x9wnpfcj5j9g7",amount: "0"},
        {address: "terra12g4sj6euv68kgx40k7mxu5xlm5sfat806umek7",amount: "0"},
        {address: deployer.key.accAddress, amount: "010000000000000"},
        ],
    mint: {
        minter: "terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k",
        cap: "420000000000000"
    },
    marketing: {
        project: "crypto11.me",
        description: "This token in meant to be used for playing gamesin crypto11 world",
        marketing: "terra1wjq02nwcv6rq4zutq9rpsyq9k08rj30rhzgvt4"
    },
}