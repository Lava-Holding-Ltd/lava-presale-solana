use crate::error::ErrorCode;
use crate::events::Contributed;
use crate::{
    PresaleConfig, ReferralData, Round, UserContribution, BASIS_POINTS,
    MAX_CONTRIBUTION_USD_PER_USER, PRESALE_SEED, ROUND_SEED, SOL_DECIMALS,
    SOL_USD_PRICE_FEED_ACCOUNT, USDC_DECIMALS, USER_CONTRIBUTION_SEED,
};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

#[derive(Accounts)]
pub struct BuyWithSol<'info> {
    pub authority: Signer<'info>,

    /// CHECK: Treasury wallet that receives funds
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        has_one = treasury @ ErrorCode::Unauthorized,
        has_one = authority @ ErrorCode::Unauthorized,
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

    #[account(address = SOL_USD_PRICE_FEED_ACCOUNT)]
    pub price_update: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<BuyWithSol>,
    token_amount: u64,
    referral: Option<ReferralData>,
) -> Result<()> {
    require!(
        !ctx.accounts.presale_config.finalized,
        ErrorCode::PresaleEnded
    );
    require_eq!(
        ctx.accounts.presale_config.current_round,
        ctx.accounts.active_round.round_id,
        ErrorCode::InvalidRoundConfig
    );

    let now = Clock::get()?.unix_timestamp;

    require_gte!(now, ctx.accounts.active_round.start_time);
    require_gte!(ctx.accounts.active_round.end_time, now);

    require_gt!(token_amount, 0);
    let price_update = &mut ctx.accounts.price_update;
    // get_price_no_older_than will fail if the price update is more than 30 seconds old
    #[cfg(feature = "devnet")]
    let maximum_age: u64 = 3000000000;
    #[cfg(not(feature = "devnet"))]
    let maximum_age: u64 = 30;
    // get_price_no_older_than will fail if the price update is for a different price feed.
    let feed_id: [u8; 32] =
        get_feed_id_from_hex("0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d")?; // SOL/USD feed

    let price_data = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;

    if ctx.accounts.user_contribution.total_contributed_usd == 0 {
        ctx.accounts.user_contribution.set_inner(UserContribution {
            user: ctx.accounts.user.key(),
            total_contributed_usd: 0,
            total_tokens_purchased: 0,
            bump: ctx.bumps.user_contribution,
        });
    }

    // Calculate LAVA token price in SOL
    // token_amount has 6 decimals (actual_tokens * 10^6)
    // token_price_usd has 6 decimals (price per 1 token * 10^6)
    // price_data.price has 8 decimals but negative exponent (SOL price in USD * 10^8)

    let sol_price_usd = price_data.price as u128;
    let token_price_usd = ctx.accounts.active_round.token_price_usd as u128;

    // Calculate total cost: (token_amount * token_price_usd) / sol_price_usd
    //
    // Formula:
    //       token_amount * token_price_usd * 10^12
    // -------------------------------------------------------
    //   sol_price_usd * 10^|exponent| * 10^(12 - |exponent|)
    let total_cost_usd = (token_amount as u128)
        .checked_mul(token_price_usd)
        .ok_or(ErrorCode::ArithmeticOverflow)?; // USD cost * 10^12

    let convert_exponent: i32 = USDC_DECIMALS as i32 * 2 + price_data.exponent;
    let convert_n = 10u128.pow(convert_exponent as u32);

    let total_sol_lamports = total_cost_usd
        .checked_div(
            sol_price_usd
                .checked_mul(convert_n) // 10^4
                .ok_or(ErrorCode::ArithmeticOverflow)?,
        )
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_mul(10u128.pow(SOL_DECIMALS as u32))
        .ok_or(ErrorCode::ArithmeticOverflow)? as u64;

    require_gt!(total_sol_lamports, 0);

    let bonus_tokens = match &referral {
        Some(ref referral_data) => token_amount
            .checked_mul(referral_data.bonus_percent as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?
            .checked_div(BASIS_POINTS as u64)
            .ok_or(ErrorCode::ArithmeticOverflow)?,
        None => 0,
    };

    msg!(
        "Calculated price: {} LAVA tokens ({} with decimals) cost {} lamports (SOL price: ${}, Token price: ${})",
        token_amount as f64 / 1_000_000.0, // Actual token amount
        token_amount, // Raw amount with 6 decimals
        total_sol_lamports,
        sol_price_usd as f64 / 100_000_000.0, // Convert from 8 decimals
        token_price_usd as f64 / 1_000_000.0  // Convert from 6 decimals
    );

    let transfer_accounts = Transfer {
        from: ctx.accounts.user.to_account_info(),
        to: ctx.accounts.treasury.to_account_info(),
    };

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_accounts,
        ),
        total_sol_lamports,
    )?;

    let contributed_amount_usd = total_cost_usd
        .checked_div(10u128.pow(6 as u32))
        .ok_or(ErrorCode::ArithmeticOverflow)? as u64;

    ctx.accounts.user_contribution.total_contributed_usd += contributed_amount_usd;
    ctx.accounts.user_contribution.total_tokens_purchased += token_amount + bonus_tokens;

    require_gte!(
        MAX_CONTRIBUTION_USD_PER_USER,
        ctx.accounts.user_contribution.total_contributed_usd,
        ErrorCode::ExceedsMaxContribution
    );

    emit!(Contributed {
        contributor: ctx.accounts.user.key(),
        stage_id: ctx.accounts.active_round.round_id,
        amount_tokens: token_amount,
        amount_referral_bonus_tokens: bonus_tokens,
        contributed_amount_usd,
        referral,
    });

    Ok(())
}
