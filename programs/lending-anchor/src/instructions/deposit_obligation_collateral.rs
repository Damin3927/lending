use crate::{
    account_data::{lending_market::LendingMarket, obligation::Obligation, reserve::Reserve},
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct DepositObligationCollateral<'info> {
    #[account(
        constraint = source_collateral.key() != deposit_reserve.collateral.supply_pubkey @ LendingError::InvalidAccountInput
    )]
    pub source_collateral: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = destination_collateral.key() == deposit_reserve.collateral.supply_pubkey @ LendingError::InvalidAccountInput
    )]
    pub destination_collateral: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = !deposit_reserve.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ReserveStale,
        constraint = deposit_reserve.config.loan_to_value_ratio != 0 @ LendingError::InvalidConfig,
    )]
    pub deposit_reserve: Box<Account<'info, Reserve>>,

    #[account(
        constraint = obligation.lending_market.key() == lending_market.key() @ LendingError::InvalidMarketOwner,
    )]
    pub obligation: Box<Account<'info, Obligation>>,

    pub lending_market: Box<Account<'info, LendingMarket>>,

    #[account(
        constraint = obligation.owner == obligation_owner.key() @ LendingError::InvalidObligationOwner,
    )]
    pub obligation_owner: Signer<'info>,

    /// CHECK:
    pub user_transfer_authority: UncheckedAccount<'info>,

    #[account(
        constraint = lending_market.token_program_id == *token_program.key @ LendingError::InvalidTokenProgram,
    )]
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositObligationCollateral<'info> {
    pub fn into_transfer_collateral_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_collateral.to_account_info(),
            to: self.destination_collateral.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_deposit_obligation_collateral(
    ctx: Context<DepositObligationCollateral>,
    collateral_amount: u64,
) -> Result<()> {
    ctx.accounts
        .obligation
        .find_or_add_collateral_to_deposits(ctx.accounts.deposit_reserve.key())?;
    ctx.accounts.obligation.last_update.mark_stale();

    transfer(
        ctx.accounts.into_transfer_collateral_ctx(),
        collateral_amount,
    )?;
    Ok(())
}
