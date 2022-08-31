import {
  Secp256k1HdWallet,
  CosmWasmClient,
  SigningCosmWasmClient,
  GasPrice,
  coin,
} from "cosmwasm";
import * as fs from "fs";
import axios from "axios";
require("dotenv").config();

const rpcEndpoint = "https://rpc.uni.juno.deuslabs.fi";

const rps_wasm = fs.readFileSync("../artifacts/rock_paper_scissors.wasm");

const sender_addr = process.env.SENDER;
const mnemonic = process.env.MNEMONIC;

const sender_addr_2 = process.env.SENDER_2;
const mnemonic_2 = process.env.MNEMONIC_2;

const contract_addr =
  "juno1f7g2u00wxkvx5npy4xay2ehc2esd9zhgf5hpctsel6sj453qe7hqtac77e";

describe("CosmWasm Tests", () => {
  /* xit("Generate a wallet", async () => {
    const wallet = await Secp256k1HdWallet.generate(12, { prefix: "juno" });

    console.log("mnemonic: ", wallet.mnemonic);
    console.log("WALLET: ", await wallet.getAccounts());
  }).timeout(50000);

   xit("Get Testnet Tokens for opponent address", async () => {
    let wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic_2, {
      prefix: "juno",
    });

    try {
      let res = await axios.post("https://faucet.uni.juno.deuslabs.fi/credit", {
        denom: "ujunox",
        address: sender_addr_2,
      });
      console.log(res);
    } catch (e) {
      console.log(e);
    }
  }).timeout(50000);

  xit("Query wallet address", async () => {
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic_2, {
      prefix: "juno",
    });

    console.log("WALLET: ", await wallet.getAccounts());
  }).timeout(50000);

  xit("Check wallet address balance", async () => {
    const client = await CosmWasmClient.connect(rpcEndpoint);

    let address_balance = await client.getBalance(sender_addr, "ujunox");

    console.log("address balance: ", address_balance);
  }).timeout(50000);

  xit("Upload contract to Juno testnet", async () => {
    let gas = GasPrice.fromString("0.025ujunox");

    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "juno",
    });

    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      { gasPrice: gas }
    );

    let res = await client.upload(sender_addr, rps_wasm, "auto");

    console.log("RES: ", res);
  }).timeout(50000); */

  xit("Instantiate the contract", async () => {
    let gas = GasPrice.fromString("0.025ujunox");
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "juno",
    });
    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      { gasPrice: gas }
    );

    let res = await client.instantiate(
      sender_addr,
      3174,
      { admin: sender_addr },
      "deposit_example",
      "auto",
      { admin: sender_addr }
    );

    console.log("RES: ", res);
  }).timeout(50000);

  xit("Start game", async () => {
    let gas = GasPrice.fromString("0.025ujunox");
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "juno",
    });
    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      { gasPrice: gas }
    );

    let host_wager = [coin(1000000, "ujunox")];

    let res = await client.execute(
      sender_addr,
      contract_addr,
      { start_game: { opponent: sender_addr_2, host_move: "Rock" } },
      "auto",
      undefined,
      host_wager
    );

    console.log("RES: ", res);
  }).timeout(50000);

  xit("Opponent responds and end game", async () => {
    let gas = GasPrice.fromString("0.025ujunox");
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic_2, {
      prefix: "juno",
    });
    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      { gasPrice: gas }
    );

    //console.log("WALLET: ", await wallet.getAccounts());

    let opp_wager = [coin(1000000, "ujunox")];

    let res = await client.execute(
      sender_addr_2,
      contract_addr,
      { opponent_response: { host: sender_addr, opp_move: "Paper" } },
      "auto",
      undefined,
      opp_wager
    );

    console.log("RES: ", res);
  }).timeout(50000);

  xit("Query for balance in contract", async () => {
    const client = await CosmWasmClient.connect(rpcEndpoint);

    let res = await client.queryContractSmart(contract_addr, {
      get_game_by_host_and_opponent: {
        host: sender_addr,
        opponent: sender_addr_2,
      },
    });

    console.log("RES: ", res.games[0].host_wager);
  }).timeout(50000);

  xit("migrate the contract", async () => {
    let gas = GasPrice.fromString("0.025ujunox");
    const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "juno",
    });
    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      { gasPrice: gas }
    );

    let res = await client.migrate(
      sender_addr,
      contract_addr,
      3098,
      {},
      "auto"
    );

    console.log("MIGRATE RES: ", res);
  }).timeout(50000);
});
