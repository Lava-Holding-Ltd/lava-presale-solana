import {
  address,
  createSolanaClient,
  createTransaction,
  getBytesEncoder,
  getProgramDerivedAddress,
  getU64Encoder,
  getU8Encoder,
  signAndSendTransactionMessageWithSigners,
  signTransactionMessageWithSigners,
} from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import * as programClient from "../clients/js/src/generated";
import { TREASURY } from "./utils";

const { rpc, sendAndConfirmTransaction, simulateTransaction } =
  createSolanaClient({ urlOrMoniker: "localnet" });

const [configAddress] = await getProgramDerivedAddress({
  programAddress: programClient.LAVA_PROGRAMS_PROGRAM_ADDRESS,
  seeds: ["presale"],
});

const config = await programClient.fetchPresaleConfig(rpc, configAddress);
const [activeStageAddress] = await getProgramDerivedAddress({
  seeds: ["stage", getU8Encoder().encode(config.data.currentStage)],
  programAddress: programClient.LAVA_PROGRAMS_PROGRAM_ADDRESS,
});
console.log(activeStageAddress);

const signer = await loadKeypairSignerFromFile("./keys/authority.json");
const user = await loadKeypairSignerFromFile();

const buy_with_sol_ix = await programClient.getBuyWithSolInstructionAsync({
  authority: signer,
  user,
  treasury: TREASURY,
  activeStage: activeStageAddress,
  refferal: null,
  tokenAmount: 20_153_000_000,
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const tx = createTransaction({
  version: "legacy",
  feePayer: user,
  instructions: [buy_with_sol_ix],
  latestBlockhash,
});

const simulation = await simulateTransaction(tx);
console.log("transaction simulation:");
console.log(simulation);

const signedTx = await signTransactionMessageWithSigners(tx);

await sendAndConfirmTransaction(signedTx);
