use crate::constants::{PRESALE_SEED, ROUND_SEED};
use crate::error::ErrorCode;
use crate::state::PresaleConfig;
use crate::{CreateRoundData, Round};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetNewRound<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ ErrorCode::Unauthorized,
        seeds = [PRESALE_SEED.as_bytes()],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,

    #[account(
        init,
        payer = authority,
        space = Round::DISCRIMINATOR.len() + Round::INIT_SPACE,
        seeds = [ROUND_SEED.as_bytes(), (presale_config.current_round + 1).to_le_bytes().as_ref()],
        bump
    )]
    pub round: Account<'info, Round>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SetNewRound>, new_round: CreateRoundData) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;
    let stage = &mut ctx.accounts.round;

    require!(
        !presale_config.finalized,
        ErrorCode::PresaleAlreadyFinalized
    );
    require!(
        presale_config.current_round < (crate::constants::MAX_STAGES - 1) as u8,
        ErrorCode::InvalidRoundConfig
    );
    require!(new_round.token_price_usd > 0, ErrorCode::InvalidRoundConfig);
    require!(
        new_round.start_time < new_round.end_time,
        ErrorCode::InvalidRoundConfig
    );

    let next_stage = presale_config.current_round + 1;
    presale_config.current_round = next_stage;
    msg!("Progressed to stage {}", next_stage);

    stage.set_inner(Round {
        round_id: next_stage,
        token_price_usd: new_round.token_price_usd,
        start_time: new_round.start_time,
        end_time: new_round.end_time,
        bump: ctx.bumps.round,
    });

    Ok(())
}
