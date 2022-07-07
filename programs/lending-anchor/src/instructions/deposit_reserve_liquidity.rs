use crate::{
    account_data::{lending_market::LendingMarket, reserve::Reserve},
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct DepositReserveLiquidity<'info> {
    pub source_liquidity: Box<Account<'info, TokenAccount>>,
    pub destination_collateral: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = reserve.lending_market == lending_market.key()
        && reserve.liquidity.supply_pubkey == reserve_liquidity_supply.key()
        && reserve.collateral.mint_pubkey == reserve_collateral_mint.key()
        && reserve.liquidity.supply_pubkey != source_liquidity.key()
        && reserve.collateral.supply_pubkey != destination_collateral.key()
    )]
    pub reserve: Box<Account<'info, Reserve>>,
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    pub lending_market: Box<Account<'info, LendingMarket>>,

    /// CHECK:
    pub lending_market_authority: UncheckedAccount<'info>,

    /// CHECK:
    pub user_transfer_authority: UncheckedAccount<'info>,

    #[account(
        constraint = lending_market.token_program_id == *token_program.key
    )]
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositReserveLiquidity<'info> {
    pub fn into_transfer_user_liquidity_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.source_liquidity.to_account_info(),
            to: self.reserve_liquidity_supply.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
    pub fn into_mint_user_collateral_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reserve_collateral_mint.to_account_info(),
            to: self.destination_collateral.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_deposit_reserve_liquidity(
    ctx: Context<DepositReserveLiquidity>,
    liquidity_amount: u64,
) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.lending_market.owner.key(),
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require_keys_eq!(
        ctx.accounts.reserve.key(),
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require!(
        !ctx.accounts
            .reserve
            .last_update
            .is_stale(Clock::get()?.slot)?,
        LendingError::ReserveStale
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
        ctx.accounts.lending_market_authority.key(),
        lending_market_authority_pubkey,
        LendingError::InvalidMarketAuthority
    );

    ctx.accounts.reserve.last_update.mark_stale();

    let collateral_amount = ctx.accounts.reserve.deposit_liquidity(liquidity_amount)?;

    transfer(
        ctx.accounts.into_transfer_user_liquidity_ctx(),
        liquidity_amount,
    )?;
    mint_to(
        ctx.accounts.into_mint_user_collateral_ctx(),
        collateral_amount,
    )?;
    Ok(())
}
