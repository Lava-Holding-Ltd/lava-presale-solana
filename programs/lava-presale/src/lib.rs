#![allow(unexpected_cfgs)]

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FyB2J5z75o5bE9Ts9McZR6inuWyzpNGCKjFgBFtWAkLm");

#[program]
pub mod lava_presale {
    use super::*;

    pub fn initialize_presale(
        ctx: Context<InitializePresale>,
        first_stage: CreateRoundData,
    ) -> Result<()> {
        initialize_presale::handler(ctx, first_stage)
    }

    pub fn finalize_presale(ctx: Context<FinalizePresale>) -> Result<()> {
        finalize_presale::handler(ctx)
    }

    pub fn set_new_round(ctx: Context<SetNewRound>, new_round: CreateRoundData) -> Result<()> {
        set_new_round::handler(ctx, new_round)
    }

    pub fn buy_with_sol(
        ctx: Context<BuyWithSol>,
        token_amount: u64,
        refferal: Option<ReferralData>,
    ) -> Result<()> {
        buy_with_sol::handler(ctx, token_amount, refferal)
    }

    pub fn buy_with_usd(
        ctx: Context<BuyWithUsd>,
        token_amount: u64,
        refferal: Option<ReferralData>,
    ) -> Result<()> {
        buy_with_usd::handler(ctx, token_amount, refferal)
    }
}
