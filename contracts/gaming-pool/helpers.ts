import axios from "axios";
import fs from "fs";
import { SigningStargateClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice, calculateFee, StdFee } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet, makeCosmoshubPath } from "@cosmjs/proto-signing";
import { Slip10RawIndex } from "@cosmjs/crypto";
import { toUtf8, toBase64 } from "@cosmjs/encoding";
import path from "path";

interface Options {
  readonly httpUrl: string;
  readonly networkId: string;
  readonly feeToken: string;
  readonly bech32prefix: string;
  readonly hdPath: readonly Slip10RawIndex[];
  readonly faucetUrl?: string;
  readonly defaultKeyFile: string;
  readonly fees: {
    upload: StdFee;
    init: StdFee;
    exec: StdFee;
  };
}

const furyaGasPrice = GasPrice.fromString("0.01ufurya");
const furyaOptions: Options = {
  httpUrl: "https://rpc.furya.cosmwasm.com", // Replace with the actual RPC endpoint of the Furya chain
  networkId: "furya-1",
  bech32prefix: "furya",
  feeToken: "ufury",
  faucetUrl: "https://faucet.furya.cosmwasm.com/credit", // Replace with the actual faucet URL
  hdPath: makeCosmoshubPath(0),
  defaultKeyFile: path.join(process.env.HOME, ".furya.key"),
  fees: {
    upload: calculateFee(1500000, furyaGasPrice),
    init: calculateFee(500000, furyaGasPrice),
    exec: calculateFee(200000, furyaGasPrice),
  },
};

// Rest of the code remains the same

// Example usage for the adapted options
const network = useOptions(furyaOptions);
const [address, client] = await network.setup("password");
const contract = CW20(client, furyaOptions.fees);

// Replace the URL with the actual URL of the Furya native bank token contract
const sourceUrl = "https://github.com/your-repo/fury-native-bank-token.wasm";
const codeId = await contract.upload(address, sourceUrl);
const initMsg = {
  // Your CW20 initialization message
};
const instance = await contract.instantiate(address, codeId, initMsg, "Your CW20 Token Name");
