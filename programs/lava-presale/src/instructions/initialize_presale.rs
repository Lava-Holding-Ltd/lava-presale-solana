use crate::constants::PRESALE_SEED;
use crate::state::PresaleConfig;
use crate::{CreateRoundData, Round, AUTHORITY, ROUND_SEED, START_ROUND_ID, USDC_MINT, USDT_MINT};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

#[derive(Accounts)]
pub struct InitializePresale<'info> {
    #[account(
        mut,
        address = AUTHORITY
    )]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = PresaleConfig::DISCRIMINATOR.len() + PresaleConfig::INIT_SPACE,
        seeds = [PRESALE_SEED.as_bytes()],
        bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,

    #[account(
        init,
        payer = authority,
        space = Round::DISCRIMINATOR.len() + Round::INIT_SPACE,
        seeds = [ROUND_SEED.as_bytes(), START_ROUND_ID.to_le_bytes().as_ref()],
        bump
    )]
    pub round: Account<'info, Round>,

    /// CHECK: Treasury wallet that will receive funds
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = treasury,
        associated_token::token_program = token_program,
    )]
    pub treasury_usdc_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = usdt_mint,
        associated_token::authority = treasury,
        associated_token::token_program = token_program,
    )]
    pub treasury_usdt_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(address = USDC_MINT)]
    pub usdc_mint: InterfaceAccount<'info, Mint>,

    #[account(address = USDT_MINT)]
    pub usdt_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePresale>, first_stage: CreateRoundData) -> Result<()> {
    let presale_config = &mut ctx.accounts.presale_config;

    presale_config.set_inner(PresaleConfig {
        authority: ctx.accounts.authority.key(),
        treasury: ctx.accounts.treasury.key(),
        current_round: START_ROUND_ID,
        finalized: false,
        total_allocated_tokens: 0,
        bump: ctx.bumps.presale_config,
    });

    let stage = &mut ctx.accounts.round;
    stage.set_inner(Round {
        round_id: START_ROUND_ID,
        start_time: first_stage.start_time,
        end_time: first_stage.end_time,
        token_price_usd: first_stage.token_price_usd,
        bump: ctx.bumps.round,
    });

    Ok(())
}
