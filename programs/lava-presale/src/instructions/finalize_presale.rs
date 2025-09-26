use crate::error::ErrorCode;
use anchor_lang::prelude::*;

use crate::{PresaleConfig, MAX_STAGES, PRESALE_SEED};

#[derive(Accounts)]
pub struct FinalizePresale<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority,
        seeds = [PRESALE_SEED.as_bytes()],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,
}

pub fn handler(ctx: Context<FinalizePresale>) -> Result<()> {
    require!(
        !ctx.accounts.presale_config.finalized,
        ErrorCode::PresaleEnded
    );
    let presale_config = &mut ctx.accounts.presale_config;

    require!(
        presale_config.current_round == MAX_STAGES as u8,
        ErrorCode::PresaleNotFinalized
    );

    presale_config.finalized = true;

    Ok(())
}
