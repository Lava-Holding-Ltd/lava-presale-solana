use anchor_lang::prelude::*;

use crate::ReferralData;

#[event]
pub struct Contributed {
    pub contributor: Pubkey,
    pub amount_tokens: u64,
    pub amount_referral_bonus_tokens: u64,
    pub contributed_amount_usd: u64,
    pub stage_id: u8,
    pub referral: Option<ReferralData>,
}
