use crate::error::ErrorCode;
use crate::events::Contributed;
use crate::{
    PresaleConfig, ReferralData, Round, UserContribution, BASIS_POINTS,
    MAX_CONTRIBUTION_USD_PER_USER, PRESALE_SEED, ROUND_SEED, USDC_DECIMALS, USDC_MINT, USDT_MINT,
    USER_CONTRIBUTION_SEED,
};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TransferChecked};

#[derive(Accounts)]
pub struct BuyWithUsd<'info> {
    pub authority: Signer<'info>,

    /// CHECK: Treasury wallet that receives funds
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        has_one = authority @ ErrorCode::Unauthorized,
        has_one = treasury @ ErrorCode::Unauthorized,
        seeds = [PRESALE_SEED.as_bytes()],
        bump = presale_config.bump
    )]
    pub presale_config: Account<'info, PresaleConfig>,

    #[account(
        seeds = [ROUND_SEED.as_bytes(), presale_config.current_round.to_le_bytes().as_ref()],
        bump = active_round.bump
    )]
    pub active_round: Account<'info, Round>,

    #[account(
        init_if_needed,
        payer = user,
        space = UserContribution::DISCRIMINATOR.len() + UserContribution::INIT_SPACE,
        seeds = [USER_CONTRIBUTION_SEED.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_contribution: Account<'info, UserContribution>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = treasury,
        associated_token::token_program = token_program
    )]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<BuyWithUsd>,
    token_amount: u64,
    referral: Option<ReferralData>,
) -> Result<()> {
    require!(
        !ctx.accounts.presale_config.finalized,
        ErrorCode::PresaleEnded
    );
    require_gt!(token_amount, 0);
    require_eq!(
        ctx.accounts.presale_config.current_round,
        ctx.accounts.active_round.round_id,
        ErrorCode::InvalidRoundConfig
    );

    let now = Clock::get()?.unix_timestamp;

    require_gte!(now, ctx.accounts.active_round.start_time);
    require_gte!(ctx.accounts.active_round.end_time, now);

    let mint = &ctx.accounts.mint;

    require!(
        mint.key() == USDC_MINT || mint.key() == USDT_MINT,
        ErrorCode::InvalidPaymentToken
    );

    let user_contribution = &mut ctx.accounts.user_contribution;
    if user_contribution.total_contributed_usd == 0 {
        user_contribution.set_inner(UserContribution {
            user: ctx.accounts.user.key(),
            total_contributed_usd: 0,
            total_tokens_purchased: 0,
            bump: ctx.bumps.user_contribution,
        });
    }
    let round = &ctx.accounts.active_round;

    let bonus_tokens = match &referral {
        Some(ref referral_data) => token_amount
            .checked_mul(referral_data.bonus_percent as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(BASIS_POINTS as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?,
        None => 0,
    };

    let token_price_usd = round.token_price_usd as u128;
    let total_cost_usd = (token_amount as u128)
        .checked_mul(token_price_usd)
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_div(10_u128.pow(USDC_DECIMALS as u32))
        .ok_or(ErrorCode::ArithmeticOverflow)? as u64;

    user_contribution.total_contributed_usd += total_cost_usd;
    user_contribution.total_tokens_purchased += token_amount + bonus_tokens;

    require_gte!(
        MAX_CONTRIBUTION_USD_PER_USER,
        user_contribution.total_contributed_usd,
        ErrorCode::ExceedsMaxContribution
    );

    let transfer_accounts = TransferChecked {
        from: ctx.accounts.user_ata.to_account_info(),
        to: ctx.accounts.treasury_ata.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );

    transfer_checked(transfer_ctx, total_cost_usd, ctx.accounts.mint.decimals)?;

    emit!(Contributed {
        contributor: ctx.accounts.user.key(),
        stage_id: ctx.accounts.active_round.round_id,
        amount_tokens: token_amount,
        amount_referral_bonus_tokens: bonus_tokens,
        contributed_amount_usd: total_cost_usd,
        referral,
    });

    Ok(())
}
