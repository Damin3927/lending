pub mod account_data;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod pyth;
pub mod utils;

use crate::{
    account_data::reserve::reserve_config::ReserveConfig,
    instructions::{
        borrow_obligation_liquidity::*, deposit_obligation_collateral::*,
        deposit_reserve_liquidity::*, init_lending_market::*, init_obligation::*, init_reserve::*,
        redeem_reserve_collateral::*, set_lending_market_owner::*,
        withdraw_obligation_collateral::*,
    },
};
use anchor_lang::prelude::*;

declare_id!("5svw6ndVHyYaUASP5njKKCMM8GiaXGTJZiycw7TGY5Y2");

#[program]
pub mod lending_anchor {
    use super::*;

    pub fn init_lending_market(
        ctx: Context<InitLendingMarket>,
        quote_currency: [u8; 32],
    ) -> Result<()> {
        process_init_lending_market(ctx, quote_currency)
    }

    pub fn set_lending_market_owner(
        ctx: Context<SetLendingMarketOwner>,
        new_owner: Pubkey,
    ) -> Result<()> {
        process_set_lending_market_owner(ctx, new_owner)
    }

    pub fn init_reserve(
        ctx: Context<InitReserve>,
        liquidity_amount: u64,
        reserve_config: ReserveConfig,
    ) -> Result<()> {
        process_init_reserve(ctx, liquidity_amount, reserve_config)
    }

    pub fn deposit_reserve_liquidity(
        ctx: Context<DepositReserveLiquidity>,
        liquidity_amount: u64,
    ) -> Result<()> {
        process_deposit_reserve_liquidity(ctx, liquidity_amount)
    }

    pub fn redeem_reserve_collateral(
        ctx: Context<RedeemReserveCollateral>,
        collateral_amount: u64,
    ) -> Result<()> {
        process_redeem_reserve_collateral(ctx, collateral_amount)
    }

    pub fn init_obligation(ctx: Context<InitObligation>) -> Result<()> {
        process_init_obligation(ctx)
    }

    pub fn deposit_obligation_collateral(
        ctx: Context<DepositObligationCollateral>,
        collateral_amount: u64,
    ) -> Result<()> {
        process_deposit_obligation_collateral(ctx, collateral_amount)
    }

    pub fn withdraw_obligation_collateral(
        ctx: Context<WithdrawObligationCollateral>,
        collateral_amount: u64,
    ) -> Result<()> {
        process_withdraw_obligation_collateral(ctx, collateral_amount)
    }

    pub fn borrow_obligation_liquidity(
        ctx: Context<BorrowObligationLiquidity>,
        liquidity_amount: u64,
    ) -> Result<()> {
        process_borrow_obligation_liquidity(ctx, liquidity_amount)
    }
}
