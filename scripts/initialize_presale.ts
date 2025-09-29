import {
  address,
  createSolanaClient,
  createTransaction,
  LAMPORTS_PER_SOL,
  signTransactionMessageWithSigners,
} from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import * as programClient from "../clients/js/src/generated";
import { USDC_MINT, USDT_MINT } from "./utils";

const { rpc, sendAndConfirmTransaction, simulateTransaction } =
  createSolanaClient({ urlOrMoniker: "devnet" });

const signer = await loadKeypairSignerFromFile("./keys/authority.json");
const treasury = address("6dvkrsE8LUGjuaR7oYfUfj1N1Y5jzorQTm1WDcXwWxHm");

const DURATION_IN_SECONDS = 6000;
const now = Math.floor(Date.now() / 1000);
const end = now + DURATION_IN_SECONDS;

const initialize_ix = await programClient.getInitializePresaleInstructionAsync({
  authority: signer,
  treasury,
  firstStage: {
    tokenPriceUsd: 11_000,
    startTime: now,
    endTime: end,
  },
  usdcMint: USDC_MINT,
  usdtMint: USDT_MINT,
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const tx = createTransaction({
  version: "legacy",
  feePayer: signer,
  instructions: [initialize_ix],
  latestBlockhash,
});

const simulation = await simulateTransaction(tx);
console.log("transaction simulation:");
console.log(simulation);

const signedTx = await signTransactionMessageWithSigners(tx);

await sendAndConfirmTransaction(signedTx);
