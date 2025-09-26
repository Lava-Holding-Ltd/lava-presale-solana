import { address } from "gill";

type Env = "devnet" | "mainnet";
const env: Env = "devnet";

export const _USDC_MAINNET_MINT = address(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
);
export const _USDT_MAINNET_MINT = address(
  "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
);
export const _USD_DEVNET_MINT = address(
  "7JUTQ4o61GTP8yvUat3vzuWcrBzL4QwCfsqRU3ve3QCV"
);

export const TREASURY = address("6dvkrsE8LUGjuaR7oYfUfj1N1Y5jzorQTm1WDcXwWxHm");

export const USDC_MINT =
  env === "devnet" ? _USD_DEVNET_MINT : _USDC_MAINNET_MINT;
export const USDT_MINT =
  env === "devnet" ? _USD_DEVNET_MINT : _USDT_MAINNET_MINT;
