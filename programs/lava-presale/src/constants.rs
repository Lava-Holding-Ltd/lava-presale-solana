use anchor_lang::prelude::*;

#[constant]
pub const PRESALE_SEED: &str = "presale";

#[constant]
pub const ROUND_SEED: &str = "stage";

#[constant]
pub const USER_CONTRIBUTION_SEED: &str = "user_contribution";

#[constant]
#[cfg(not(feature = "devnet"))]
pub const USDC_MINT: Pubkey =
    Pubkey::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // 7JUTQ4o61GTP8yvUat3vzuWcrBzL4QwCfsqRU3ve3QCV devnet
#[cfg(feature = "devnet")]
pub const USDC_MINT: Pubkey =
    Pubkey::from_str_const("7JUTQ4o61GTP8yvUat3vzuWcrBzL4QwCfsqRU3ve3QCV");

#[constant]
#[cfg(not(feature = "devnet"))]
pub const USDT_MINT: Pubkey =
    Pubkey::from_str_const("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"); // 7JUTQ4o61GTP8yvUat3vzuWcrBzL4QwCfsqRU3ve3QCV devnet
#[cfg(feature = "devnet")]
pub const USDT_MINT: Pubkey =
    Pubkey::from_str_const("7JUTQ4o61GTP8yvUat3vzuWcrBzL4QwCfsqRU3ve3QCV");

#[constant]
pub const SOL_USD_PRICE_FEED_ACCOUNT: Pubkey =
    Pubkey::from_str_const("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

#[constant]
pub const AUTHORITY: Pubkey = Pubkey::from_str_const("C4o2Cbe6RNSCyTqtLnkQZoVimapXK5xfpjRnWmEoxVeo");

pub const MAX_STAGES: usize = 10;

pub const MAX_CONTRIBUTION_USD_PER_USER: u64 = 50_000 * (10_u64.pow(USDC_DECIMALS as u32));

pub const BASIS_POINTS: usize = 10_000; // 100 %

pub const MAX_BASIS_POINTS: usize = 1_000; // 10 %

pub const START_ROUND_ID: u8 = 1;

pub const MAX_TOKEN_CAP: u64 = 330_000_000 * (10_u64.pow(LAVA_DECIMALS as u32));

pub const USDC_DECIMALS: u8 = 6;
pub const USDT_DECIMALS: u8 = 6;
pub const LAVA_DECIMALS: u8 = 6;
pub const SOL_DECIMALS: u8 = 9;
