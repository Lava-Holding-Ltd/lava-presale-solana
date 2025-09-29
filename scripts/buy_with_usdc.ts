import {
  address,
  createSolanaClient,
  createTransaction,
  getProgramDerivedAddress,
  getU8Encoder,
  signTransactionMessageWithSigners,
} from "gill";
import { loadKeypairSignerFromFile } from "gill/node";
import * as programClient from "../clients/js/src/generated";
import { TREASURY, USDC_MINT } from "./utils";

const { rpc, sendAndConfirmTransaction, simulateTransaction } =
  createSolanaClient({ urlOrMoniker: "devnet" });

const [configAddress] = await getProgramDerivedAddress({
  programAddress: programClient.LAVA_PRESALE_PROGRAM_ADDRESS,
  seeds: ["presale"],
});

const config = await programClient.fetchPresaleConfig(rpc, configAddress);
console.log(config.data.currentRound);
const [activeStageAddress] = await getProgramDerivedAddress({
  seeds: ["stage", getU8Encoder().encode(config.data.currentRound)],
  programAddress: programClient.LAVA_PRESALE_PROGRAM_ADDRESS,
});
console.log(activeStageAddress);

const signer = await loadKeypairSignerFromFile("./keys/authority.json");
const user = await loadKeypairSignerFromFile();

const buy_with_usdc_ix = await programClient.getBuyWithUsdInstructionAsync({
  authority: signer,
  user,
  treasury: TREASURY,
  activeRound: activeStageAddress,
  refferal: null,
  tokenAmount: 10000,
  mint: USDC_MINT,
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const tx = createTransaction({
  version: "legacy",
  feePayer: user,
  instructions: [buy_with_usdc_ix],
  latestBlockhash,
});

const simulation = await simulateTransaction(tx);
console.log("transaction simulation:");
console.log(simulation);

const signedTx = await signTransactionMessageWithSigners(tx);

await sendAndConfirmTransaction(signedTx);
