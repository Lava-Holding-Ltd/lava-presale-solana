use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PresaleConfig {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub current_round: u8,
    pub finalized: bool,
    pub bump: u8,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, Debug)]
pub struct CreateRoundData {
    pub token_price_usd: u64, // Price per token in USD (6 decimals)
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct ReferralData {
    pub code: String,
    pub bonus_percent: u16,
    pub ref_type: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Round {
    pub round_id: u8,
    pub token_price_usd: u64, // Price per token in USD (6 decimals)
    pub start_time: i64,
    pub end_time: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserContribution {
    pub user: Pubkey,
    pub total_contributed_usd: u64, // Total contributed in USD (6 decimals)
    pub total_tokens_purchased: u64,
    pub bump: u8,
}

impl Round {
    pub fn is_active(&self) -> bool {
        let now = Clock::get().unwrap().unix_timestamp;
        now >= self.start_time && now <= self.end_time
    }
}
