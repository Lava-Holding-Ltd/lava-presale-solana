# Lava Presale Program - Audit Documentation

## Executive Summary

This document provides comprehensive technical documentation for the Lava Presale Program, a Solana-based smart contract implementing a multi-round token presale system. The program facilitates token sales through SOL and USD stablecoin payments with backend-controlled authorization.

**Program ID**: `FyB2J5z75o5bE9Ts9McZR6inuWyzpNGCKjFgBFtWAkLm`

## Architecture Overview

### Core Components

1. **PresaleConfig Account**: Global presale state and configuration
2. **Round Account**: Individual round parameters and timing
3. **UserContribution Account**: Per-user contribution tracking via PDA
4. **External Authority**: Backend-controlled signer for transaction validation

### Security Model

- **Backend Authority Control**: Hardcoded authority (`9YS6irKCxBYmYX28ifG25c8CvrKi4cmNDDptZvBxELF`) required for all administrative operations
- **External Signer Validation**: All purchase transactions require backend authority signature
- **PDA-based Tracking**: User contributions tracked via deterministic PDAs to prevent double-counting
- **Direct Treasury Transfers**: No custodial model - funds directly transferred to treasury wallet

## Program Instructions

### 1. `initialize_presale`

**Purpose**: Initialize the presale system with the first round configuration.

**Authority**: Backend only (hardcoded `AUTHORITY` constant)

**Key Validations**:
- Only callable by authorized backend signer
- Creates PresaleConfig and first Round account
- Sets up treasury token accounts for USDC/USDT
- Initializes with round ID 1

**Account Structure**:
```rust
pub struct InitializePresale<'info> {
    #[account(address = AUTHORITY)]
    pub authority: Signer<'info>,

    #[account(init, seeds = [PRESALE_SEED.as_bytes()], bump)]
    pub presale_config: Account<'info, PresaleConfig>,

    #[account(init, seeds = [ROUND_SEED.as_bytes(), START_ROUND_ID.to_le_bytes().as_ref()], bump)]
    pub round: Account<'info, Round>,

    pub treasury: UncheckedAccount<'info>,
    // ... token accounts
}
```

### 2. `finalize_presale`

**Purpose**: Mark the presale as complete when all rounds are finished.

**Authority**: Backend only

**Key Validations**:
- Only callable by authorized backend signer
- Requires current_round == MAX_STAGES (10)
- Cannot finalize already finalized presale
- Sets `finalized` flag to true

### 3. `set_new_round`

**Purpose**: Create a new round with updated pricing and timing parameters.

**Authority**: Backend only

**Key Validations**:
- Only callable by authorized backend signer
- Cannot exceed MAX_STAGES limit (10 rounds maximum)
- Validates round timing (start_time < end_time)
- Validates token price > 0
- Increments current_round counter

**Business Logic**:
- Creates new Round PDA with incremented round ID
- Updates PresaleConfig.current_round
- Backend determines all pricing, timing, and supply parameters

### 4. `buy_with_sol`

**Purpose**: Allow users to purchase tokens using SOL with real-time price conversion.

**Authority**: Requires both user and backend authority signatures

**Key Features**:
- **Dual Signature Requirement**: Both user and backend authority must sign
- **Real-time Price Feeds**: Uses Pyth oracle for SOL/USD conversion
- **Referral System**: Optional referral bonus calculated in basis points
- **Contribution Tracking**: Updates user's total USD contribution and tokens purchased

**Critical Validations**:
```rust
// Authority validation
#[account(has_one = authority @ ErrorCode::Unauthorized)]
pub presale_config: Account<'info, PresaleConfig>,

// Time validation
require_gte!(now, ctx.accounts.active_round.start_time);
require_gte!(ctx.accounts.active_round.end_time, now);

// Contribution limits
require_gte!(
    MAX_CONTRIBUTION_USD_PER_USER,
    ctx.accounts.user_contribution.total_contributed_usd,
    ErrorCode::ExceedsMaxContribution
);
```

**Price Calculation Logic**:
```rust
// SOL price from Pyth oracle (8 decimals, negative exponent)
let sol_price_usd = price_data.price as u128;
let token_price_usd = ctx.accounts.active_round.token_price_usd as u128;

// Calculate total cost in lamports
let total_cost_usd = (token_amount as u128)
    .checked_mul(token_price_usd)
    .ok_or(ErrorCode::ArithmeticOverflow)?;

let total_sol_lamports = total_cost_usd
    .checked_div(sol_price_usd.checked_mul(convert_n))
    .ok_or(ErrorCode::ArithmeticOverflow)?
    .checked_mul(10u128.pow(SOL_DECIMALS as u32)) as u64;
```

### 5. `buy_with_usd`

**Purpose**: Allow users to purchase tokens using USDC/USDT stablecoins.

**Authority**: Requires both user and backend authority signatures

**Key Features**:
- **Stablecoin Support**: Accepts USDC or USDT only (validated against hardcoded mints)
- **Simpler Pricing**: Direct USD calculation without oracle dependency
- **Referral System**: Same bonus calculation as SOL purchases
- **Token Transfer**: Uses SPL token transfer_checked for precision

**Critical Validations**:
```rust
// Token validation
require!(
    mint.key() == USDC_MINT || mint.key() == USDT_MINT,
    ErrorCode::InvalidPaymentToken
);

// Price calculation (simpler than SOL)
let total_cost_usd = (token_amount as u128)
    .checked_mul(token_price_usd)
    .checked_div(10_u128.pow(USDC_DECIMALS as u32)) as u64;
```

## State Account Definitions

### PresaleConfig
```rust
pub struct PresaleConfig {
    pub authority: Pubkey,     // Backend authority pubkey
    pub treasury: Pubkey,      // Treasury wallet receiving funds
    pub current_round: u8,     // Current active round (1-10)
    pub finalized: bool,       // Presale completion status
    pub bump: u8,             // PDA bump seed
}
```

### Round
```rust
pub struct Round {
    pub round_id: u8,          // Round identifier (1-10)
    pub token_price_usd: u64,  // Price per token (6 decimals)
    pub start_time: i64,       // Unix timestamp start
    pub end_time: i64,         // Unix timestamp end
    pub bump: u8,              // PDA bump seed
}
```

### UserContribution
```rust
pub struct UserContribution {
    pub user: Pubkey,                    // User wallet address
    pub total_contributed_usd: u64,      // Total USD contributed (6 decimals)
    pub total_tokens_purchased: u64,     // Total tokens purchased (6 decimals)
    pub bump: u8,                        // PDA bump seed
}
```

## Security Analysis

### Access Control
- **Centralized Authority**: Single hardcoded backend authority controls all administrative functions
- **Dual Signature Model**: Purchase transactions require both user and backend signatures
- **No Upgrade Authority**: Program is immutable once deployed

### Oracle Integration
- **Pyth Price Feeds**: SOL/USD pricing via Pyth oracle
- **Staleness Protection**: Maximum 30-second price age (30 minutes on devnet)
- **Feed Validation**: Hardcoded feed ID prevents oracle manipulation

### Financial Controls
- **Contribution Limits**: $50,000 USD maximum per user across all rounds
- **Direct Transfers**: No program custody - funds go directly to treasury
- **Overflow Protection**: Comprehensive arithmetic overflow checks

### PDA Security
- **Deterministic Addresses**: All PDAs use predictable seeds
- **Bump Validation**: Proper bump seed storage and validation
- **Ownership Checks**: Account ownership validated via has_one constraints

## Constants and Configuration

### Network Configuration
```rust
// Mainnet vs Devnet mint addresses
#[cfg(not(feature = "devnet"))]
pub const USDC_MINT: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
#[cfg(not(feature = "devnet"))]
pub const USDT_MINT: Pubkey = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

// Hardcoded backend authority
pub const AUTHORITY: Pubkey = "9YS6irKCxBYmYX28ifG25c8CvrKi4cmNDDptZvBxELF";

// SOL/USD Pyth price feed
pub const SOL_USD_PRICE_FEED_ACCOUNT: Pubkey = "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE";
```

### Business Parameters
```rust
pub const MAX_STAGES: usize = 10;                              // Maximum presale rounds
pub const MAX_CONTRIBUTION_USD_PER_USER: u64 = 50_000_000_000; // $50k with 6 decimals
pub const BASIS_POINTS: usize = 10_000;                        // 100% = 10,000 basis points
pub const START_ROUND_ID: u8 = 1;                             // First round identifier
```

### Token Decimals
```rust
pub const USDC_DECIMALS: u8 = 6;
pub const USDT_DECIMALS: u8 = 6;
pub const LAVA_DECIMALS: u8 = 6;
pub const SOL_DECIMALS: u8 = 9;
```

## Event Logging

The program emits detailed events for all purchases:

```rust
#[event]
pub struct Contributed {
    pub contributor: Pubkey,                    // User wallet
    pub amount_tokens: u64,                     // Base tokens purchased
    pub amount_referral_bonus_tokens: u64,      // Bonus tokens from referral
    pub contributed_amount_usd: u64,            // USD value contributed
    pub stage_id: u8,                          // Round ID
    pub referral: Option<ReferralData>,         // Referral information
}
```

## Error Handling

Comprehensive error codes cover all failure scenarios:

```rust
#[error_code]
pub enum ErrorCode {
    PresaleNotStarted,        // Round not yet active
    PresaleEnded,            // Round or presale finished
    InvalidRoundConfig,       // Invalid round parameters
    ExceedsMaxContribution,   // User contribution limit exceeded
    ArithmeticOverflow,       // Mathematical operation overflow
    Unauthorized,            // Invalid authority signature
    InvalidPaymentToken,      // Unsupported payment token
    // ... additional error codes
}
```

## Scripts

### Deployment Scripts
- `initialize_presale.ts`: Presale initialization with first round
- `set_stage.ts`: Create new rounds with updated parameters
- `buy_with_sol.ts`: SOL purchase transaction examples
- `buy_with_usdc.ts`/`buy_with_usdt.ts`: Stablecoin purchase examples
- `finalize_presale.ts`: Presale completion

### Client Generation
- Uses Codama for TypeScript client generation
- Generated clients in `clients/js/src/generated/`
- Type-safe instruction builders and account fetchers
