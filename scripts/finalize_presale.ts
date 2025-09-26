import {
  address,
  airdropFactory,
  createSolanaClient,
  createTransaction,
  lamports,
  LAMPORTS_PER_SOL,
  signTransactionMessageWithSigners,
} from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import * as programClient from "../clients/js/src/generated";

const {
  rpc,
  sendAndConfirmTransaction,
  simulateTransaction,
  rpcSubscriptions,
} = createSolanaClient({ urlOrMoniker: "localnet" });

const signer = await loadKeypairSignerFromFile("./keys/authority.json");

const finalize_ix = await programClient.getFinalizePresaleInstructionAsync({
  authority: signer,
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const tx = createTransaction({
  version: "legacy",
  feePayer: signer,
  instructions: [finalize_ix],
  latestBlockhash,
});

const simulation = await simulateTransaction(tx);
console.log("transaction simulation:");
console.log(simulation);

const signedTx = await signTransactionMessageWithSigners(tx);

await sendAndConfirmTransaction(signedTx);
