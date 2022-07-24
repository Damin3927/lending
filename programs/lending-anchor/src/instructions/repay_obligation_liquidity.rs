use crate::{
    account_data::{
        lending_market::LendingMarket,
        obligation::Obligation,
        reserve::{CalculateRepayResult, Reserve},
    },
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct RepayObligationLiquidity<'info> {
    pub source_liquidity: Box<Account<'info, TokenAccount>>,
    pub destination_liquidity: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = repay_reserve.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = repay_reserve.liquidity.supply_pubkey != source_liquidity.key() @ LendingError::InvalidAccountInput,
        constraint = repay_reserve.liquidity.supply_pubkey == destination_liquidity.key() @ LendingError::InvalidAccountInput,
        constraint = !repay_reserve.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ReserveStale,
    )]
    pub repay_reserve: Box<Account<'info, Reserve>>,

    #[account(
        constraint = obligation.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = !obligation.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ObligationStale,
    )]
    pub obligation: Box<Account<'info, Obligation>>,

    #[account(
        constraint = lending_market.token_program_id == token_program.key() @ LendingError::InvalidTokenProgram,
    )]
    pub lending_market: Box<Account<'info, LendingMarket>>,

    /// CHECK:
    pub user_transfer_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RepayObligationLiquidity<'info> {
    fn into_transfer_liquidity_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.destination_liquidity.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_repay_obligation_liquidity(
    ctx: Context<RepayObligationLiquidity>,
    liquidity_amount: u64,
) -> Result<()> {
    require_neq!(liquidity_amount, 0, LendingError::InvalidAmount);

    let (liquidity, liquidity_index) = ctx
        .accounts
        .obligation
        .find_liquidity_in_borrows(ctx.accounts.repay_reserve.key())?;
    require_neq!(
        liquidity.borrowed_amount_wads,
        0,
        LendingError::ObligationLiquidityEmpty
    );

    let CalculateRepayResult {
        settle_amount,
        repay_amount,
    } = ctx
        .accounts
        .repay_reserve
        .calculate_repay(liquidity_amount, liquidity.borrowed_amount_wads)?;
    require_neq!(repay_amount, 0, LendingError::RepayTooSmall);

    ctx.accounts
        .repay_reserve
        .liquidity
        .repay(repay_amount, settle_amount)?;
    ctx.accounts.repay_reserve.last_update.mark_stale();

    ctx.accounts
        .obligation
        .repay(settle_amount, liquidity_index)?;
    ctx.accounts.obligation.last_update.mark_stale();

    transfer(ctx.accounts.into_transfer_liquidity_ctx(), repay_amount)?;

    Ok(())
}
