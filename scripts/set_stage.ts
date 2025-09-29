import {
  address,
  airdropFactory,
  createSolanaClient,
  createTransaction,
  getProgramDerivedAddress,
  getU8Encoder,
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
} = createSolanaClient({ urlOrMoniker: "devnet" });

const [configAddress] = await getProgramDerivedAddress({
  programAddress: programClient.LAVA_PRESALE_PROGRAM_ADDRESS,
  seeds: ["presale"],
});

const config = await programClient.fetchPresaleConfig(rpc, configAddress);

const [newStageAddress] = await getProgramDerivedAddress({
  seeds: ["stage", getU8Encoder().encode(config.data.currentRound + 1)],
  programAddress: programClient.LAVA_PRESALE_PROGRAM_ADDRESS,
});

const signer = await loadKeypairSignerFromFile("./keys/authority.json");

const DURATION_IN_SECONDS = 10000;
const now = Math.floor(Date.now() / 1000);
const end = now + DURATION_IN_SECONDS;

const set_stage_ix = await programClient.getSetNewRoundInstructionAsync({
  authority: signer,
  round: newStageAddress,
  newRound: {
    tokenPriceUsd: 12_250,
    startTime: now,
    endTime: end,
  },
});

const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

const tx = createTransaction({
  version: "legacy",
  feePayer: signer,
  instructions: [set_stage_ix],
  latestBlockhash,
});

const simulation = await simulateTransaction(tx);
console.log("transaction simulation:");
console.log(simulation);

const signedTx = await signTransactionMessageWithSigners(tx);

await sendAndConfirmTransaction(signedTx);
