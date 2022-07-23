use crate::{
    account_data::{
        lending_market::LendingMarket,
        obligation::Obligation,
        reserve::{CalculateBorrowResult, Reserve},
    },
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct BorrowObligationLiquidity<'info> {
    #[account(
        constraint = source_liquidity.key() == borrow_reserve.liquidity.supply_pubkey @ LendingError::InvalidAccountInput,
    )]
    pub source_liquidity: Account<'info, TokenAccount>,

    #[account(
        constraint = destination_liuqidity.key() != borrow_reserve.liquidity.supply_pubkey @ LendingError::InvalidAccountInput,
    )]
    pub destination_liuqidity: Account<'info, TokenAccount>,

    #[account(
        constraint = borrow_reserve.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = !borrow_reserve.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ReserveStale,
    )]
    pub borrow_reserve: Account<'info, Reserve>,

    #[account(
        constraint = borrow_reserve_liquidity_fee_receiver.key() == borrow_reserve.liquidity.fee_receiver @ LendingError::InvalidAccountInput,
    )]
    pub borrow_reserve_liquidity_fee_receiver: Account<'info, TokenAccount>,

    #[account(
        constraint = obligation.lending_market.key() == lending_market.key() @ LendingError::InvalidAccountInput,
        constraint = obligation.owner.key() == obligation_owner.key() @ LendingError::InvalidObligationOwner,
        constraint = !obligation.last_update.is_stale(Clock::get()?.slot)? @ LendingError::ObligationStale,
        constraint = !obligation.deposits.is_empty() @ LendingError::ObligatinoDepositsEmpty,
        constraint = obligation.deposited_value != 0 @ LendingError::ObligationDepositsZero,
    )]
    pub obligation: Account<'info, Obligation>,
    pub lending_market: Account<'info, LendingMarket>,
    /// CHECK:
    pub lending_market_authority: UncheckedAccount<'info>,
    pub obligation_owner: Signer<'info>,
    /// CHECK:
    pub host_fee_receiver: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> BorrowObligationLiquidity<'info> {
    fn into_transfer_host_fee_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.host_fee_receiver.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_owner_fee_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.borrow_reserve_liquidity_fee_receiver.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_liquidity_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.destination_liuqidity.to_account_info(),
            authority: self.lending_market_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_borrow_obligation_liquidity(
    ctx: Context<BorrowObligationLiquidity>,
    liquidity_amount: u64,
) -> Result<()> {
    require_neq!(liquidity_amount, 0, LendingError::InvalidAmount);

    require_keys_eq!(
        *ctx.accounts.lending_market.to_account_info().owner,
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require_keys_eq!(
        *ctx.accounts.borrow_reserve.to_account_info().owner,
        *ctx.program_id,
        LendingError::InvalidAccountOwner
    );
    require_keys_eq!(
        *ctx.accounts.obligation.to_account_info().owner,
        *ctx.program_id,
        LendingError::InvalidAccountOwner
    );

    let lending_market_authority_pubkey = Pubkey::create_program_address(
        &[
            ctx.accounts.lending_market.key().as_ref(),
            &[ctx.accounts.lending_market.bump_seed],
        ],
        ctx.program_id,
    )
    .map_err(|_| LendingError::PubkeyError)?;
    require_keys_eq!(
        lending_market_authority_pubkey,
        ctx.accounts.lending_market_authority.key(),
        LendingError::InvalidMarketAuthority
    );
    let remaining_borrow_value = ctx.accounts.obligation.remaining_borrow_value()?;
    require_neq!(remaining_borrow_value, 0, LendingError::BorrowTooLarge);

    let CalculateBorrowResult {
        borrow_amount,
        receive_amount,
        borrow_fee,
        host_fee,
    } = ctx
        .accounts
        .borrow_reserve
        .calculate_borrow(liquidity_amount, remaining_borrow_value)?;
    require_neq!(receive_amount, 0, LendingError::BorrowTooSmall);

    ctx.accounts
        .borrow_reserve
        .liquidity
        .borrow_(borrow_amount)?;
    ctx.accounts.borrow_reserve.last_update.mark_stale();

    ctx.accounts
        .obligation
        .find_or_add_liquidity_to_borrows(ctx.accounts.borrow_reserve.key())?
        .borrow(borrow_amount)?;
    ctx.accounts.obligation.last_update.mark_stale();

    // fee transfers
    let mut owner_fee = borrow_fee;
    if host_fee > 0 {
        owner_fee = owner_fee
            .checked_sub(host_fee)
            .ok_or(LendingError::MathOverflow)?;
        transfer(ctx.accounts.into_transfer_host_fee_ctx(), host_fee)?;
    }
    if owner_fee > 0 {
        transfer(ctx.accounts.into_transfer_owner_fee_ctx(), owner_fee)?;
    }

    transfer(ctx.accounts.into_transfer_liquidity_ctx(), receive_amount)?;

    Ok(())
}
