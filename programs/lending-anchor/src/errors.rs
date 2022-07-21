use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
    #[msg("Instruction Unpack Error")]
    InstructionUnpackError,

    #[msg("Market owner is invalid")]
    InvalidMarketOwner,

    #[msg("Input amount is invalid")]
    InvalidAmount,

    #[msg("Input config value is invalid")]
    InvalidConfig,

    #[msg("Reserve must be initialized with liquidity")]
    ReserveNotInitializedWithLiquidity,

    #[msg("Invalid account input")]
    InvalidAccountInput,

    #[msg("Input oracle config is invalid")]
    InvalidOracleConfig,

    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Pubkey error")]
    PubkeyError,

    #[msg("Market authority is invalid")]
    InvalidMarketAuthority,

    #[msg("Insufficient liquidity available")]
    InsufficientLiquidity,

    #[msg("Stale Reserve")]
    ReserveStale,

    #[msg("Invalid Token Program ID")]
    InvalidTokenProgram,

    #[msg("Obligation Reserve accounts exceeds the limit")]
    ObligationReserveLimit,

    #[msg("Stale Obligation")]
    ObligationStale,

    #[msg("Obligation deposits is empty")]
    ObligatinoDepositsEmpty,

    #[msg("Obligation deposit amount is zero")]
    ObligationDepositsZero,

    #[msg("Deposited Obligation Collateral is empty")]
    ObligationCollateralEmpty,

    #[msg("Obligation collatateral is invalid")]
    InvalidObligationCollateral,

    #[msg("Withdraw amount is too large")]
    WithdrawTooLarge,

    #[msg("Withdraw amount is too small")]
    WithdrawTooSmall,
}
