use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Presale has not started yet")]
    PresaleNotStarted,
    #[msg("Presale has already ended")]
    PresaleEnded,
    #[msg("Invalid round configuration")]
    InvalidRoundConfig,
    #[msg("Contribution amount is below minimum")]
    BelowMinContribution,
    #[msg("Contribution amount exceeds maximum per wallet")]
    ExceedsMaxContribution,
    #[msg("Stage token supply exhausted")]
    StageSupplyExhausted,
    #[msg("Global hard cap reached")]
    HardCapReached,
    #[msg("Soft cap not reached, refunds available")]
    SoftCapNotReached,
    #[msg("User has no contributions to refund")]
    NoContributionsToRefund,
    #[msg("Refunds not available yet")]
    RefundsNotAvailable,
    #[msg("Invalid payment token")]
    InvalidPaymentToken,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Presale is not finalized")]
    PresaleNotFinalized,
    #[msg("Presale already finalized")]
    PresaleAlreadyFinalized,
    #[msg("Presale is currently paused")]
    PresalePaused,
    #[msg("Presale is not paused")]
    PresaleNotPaused,
    #[msg("Round is not active")]
    RoundNotActive,
}
