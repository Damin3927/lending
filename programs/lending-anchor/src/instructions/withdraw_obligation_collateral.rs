use crate::{
    account_data::{lending_market::LendingMarket, obligation::Obligation, reserve::Reserve},
    errors::LendingError,
    math::{common::Decimal, rate::Rate},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct WithdrawObligationCollateral<'info> {
    pub source_collateral: Box<Account<'info, TokenAccount>>,

    pub destination_collateral: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = withdraw_reserve.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = withdraw_reserve.collateral.supply_pubkey == source_collateral.key() @ LendingError::InvalidAccountInput,
        constraint = withdraw_reserve.collateral.supply_pubkey != destination_collateral.key() @ LendingError::InvalidAccountInput,
        constraint = !withdraw_reserve.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ReserveStale
    )]
    pub withdraw_reserve: Box<Account<'info, Reserve>>,

    #[account(
        constraint = obligation.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = obligation.owner == obligation_owner.key() @ LendingError::InvalidAccountInput,
        constraint = !obligation.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ObligationStale
    )]
    pub obligation: Box<Account<'info, Obligation>>,

    #[account(
        constraint = lending_market.token_program_id == token_program.key() @ LendingError::InvalidTokenProgram
    )]
    pub lending_market: Box<Account<'info, LendingMarket>>,

    /// CHECK:
    #[account(
        seeds = [lending_market.key().as_ref()],
        bump = lending_market.bump_seed,
    )]
    pub lending_market_authority: UncheckedAccount<'info>,

    pub obligation_owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

impl<'info> WithdrawObligationCollateral<'info> {
    fn into_transfer_collateral_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_collateral.to_account_info(),
            to: self.destination_collateral.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_withdraw_obligation_collateral(
    ctx: Context<WithdrawObligationCollateral>,
    collateral_amount: u64,
) -> Result<()> {
    let (collateral, collateral_index) = ctx
        .accounts
        .obligation
        .find_collateral_index_in_deposits(ctx.accounts.withdraw_reserve.key())?;
    require_neq!(
        collateral.deposited_amount,
        0,
        LendingError::ObligationCollateralEmpty
    );

    let withdraw_amount = if ctx.accounts.obligation.borrows.is_empty() {
        if collateral_amount == u64::MAX {
            collateral.deposited_amount
        } else {
            collateral.deposited_amount.min(collateral_amount)
        }
    } else if ctx.accounts.obligation.deposited_value == 0 {
        return Err(LendingError::ObligationDepositsZero.into());
    } else {
        let max_withdraw_value = ctx
            .accounts
            .obligation
            .max_withdraw_value(Rate::from_percent(
                ctx.accounts.withdraw_reserve.config.loan_to_value_ratio,
            ))?;
        require_neq!(max_withdraw_value, 0, LendingError::WithdrawTooLarge);

        let withdraw_amount = if collateral_amount == u64::MAX {
            let withdraw_value = max_withdraw_value.min(collateral.market_value);
            let withdraw_pct = withdraw_value
                .checked_div(collateral.market_value as u128)
                .ok_or(LendingError::MathOverflow)?;
            withdraw_pct
                .checked_mul(collateral.deposited_amount as u128)
                .ok_or(LendingError::MathOverflow)?
                .try_floor_u64()?
                .min(collateral.deposited_amount)
        } else {
            let withdraw_amount = collateral_amount.min(collateral.deposited_amount);
            let withdraw_pct = withdraw_amount
                .checked_div(collateral.deposited_amount)
                .ok_or(LendingError::MathOverflow)?;
            let withdraw_value = collateral
                .market_value
                .checked_mul(withdraw_pct as u128)
                .ok_or(LendingError::MathOverflow)?;
            require_gte!(
                max_withdraw_value,
                withdraw_value,
                LendingError::WithdrawTooLarge
            );
            withdraw_amount
        };
        require_neq!(withdraw_amount, 0, LendingError::WithdrawTooSmall);

        withdraw_amount
    };

    ctx.accounts
        .obligation
        .withdraw(withdraw_amount, collateral_index)?;
    ctx.accounts.obligation.last_update.mark_stale();

    transfer(ctx.accounts.into_transfer_collateral_ctx(), withdraw_amount)?;

    Ok(())
}
