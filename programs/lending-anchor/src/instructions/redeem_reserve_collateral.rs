use crate::{
    account_data::{lending_market::LendingMarket, reserve::Reserve},
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{
    burn, transfer, Burn, Burn, Mint, Token, TokenAccount, Transfer, Transfer,
};

#[derive(Accounts)]
pub struct RedeemReserveCollateral<'info> {
    pub source_collateral: Box<Account<'info, TokenAccount>>,
    pub destination_liquidity: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = reserve.lending_market == lending_market.key()
        && reserve.collateral.mint_pubkey == reserve_collateral_mint.key()
        && reserve.collateral.supply_pubkey != source_collateral.key()
        && reserve.liquidity.supply_pubkey == reserve_liquidity_supply.key()
        && reserve.liquidity.supply_pubkey != destination_liquidity.key()
    )]
    pub reserve: Box<Account<'info, Reserve>>,
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
    pub lending_market: Box<Account<'info, LendingMarket>>,

    /// CHECK:
    pub lending_market_authority: UncheckedAccount<'info>,
    pub user_transfer_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> RedeemReserveCollateral<'info> {
    pub fn into_burn_user_collateral_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.reserve_collateral_mint.to_account_info(),
            from: self.source_collateral.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn into_transfer_user_liquidity_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.reserve_liquidity_supply.to_account_info(),
            to: self.destination_liquidity.to_account_info(),
            authority: self.user_transfer_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn process_redeem_reserve_collateral(
    ctx: Context<RedeemReserveCollateral>,
    collateral_amount: u64,
) -> Result<()> {
    require_gt!(collateral_amount, 0);
    require_keys_eq!(
        ctx.accounts.lending_market.owner.key(),
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require_keys_eq!(
        ctx.accounts.reserve.to_account_info().owner.key(),
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

    let liquidity_amount = ctx.accounts.reserve.redeem_collateral(collateral_amount)?;
    reserve.last_update.mark_stale();

    burn(
        ctx.accounts.into_burn_user_collateral_ctx(),
        collateral_amount,
    )?;
    transfer(ctx.accounts.into_transfer_user_liquidity_ctx(), amount)?;

    Ok(())
}
