pub mod collateral_exchange_rate;
pub mod reserve_collateral;
pub mod reserve_config;
pub mod reserve_fees;
pub mod reserve_liquidity;

use crate::{
    account_data::{
        last_update::LastUpdate,
        reserve::{
            collateral_exchange_rate::CollateralExchangeRate,
            reserve_collateral::ReserveCollateral, reserve_config::ReserveConfig,
            reserve_fees::FeeCalculation, reserve_liquidity::ReserveLiquidity,
        },
    },
    constants::PROGRAM_VERSION,
    errors::LendingError,
    math::common::Decimal,
    utils::byte_length::ByteLength,
};
use anchor_lang::{prelude::*, solana_program::clock::Slot};

#[account]
pub struct Reserve {
    pub version: u8,
    pub last_update: LastUpdate,

    /// Lending market address
    pub lending_market: Pubkey,

    /// Reserve liquidity
    pub liquidity: ReserveLiquidity,

    /// Reserve collateral
    pub collateral: ReserveCollateral,

    /// Reserve configuration values
    pub config: ReserveConfig,
}

impl ByteLength for Reserve {
    const LEN: usize = 1 + LastUpdate::LEN + 32 + ReserveLiquidity::LEN + ReserveCollateral::LEN;
}

pub struct InitReserveParams {
    /// Last slot when supply and rates updated
    pub current_slot: Slot,

    /// Lending market address
    pub lending_market: Pubkey,

    /// Reserve liquidity
    pub liquidity: ReserveLiquidity,

    /// Reserve collateral
    pub collateral: ReserveCollateral,

    /// Reserve configuration values
    pub config: ReserveConfig,
}

#[derive(Debug)]
pub struct CalculateBorrowResult {
    pub borrow_amount: u128,
    pub receive_amount: u64,
    pub borrow_fee: u64,
    pub host_fee: u64,
}

#[derive(Debug)]
pub struct CalculateRepayResult {
    pub settle_amount: u128,
    pub repay_amount: u64,
}

impl Reserve {
    pub fn init(&mut self, params: InitReserveParams) {
        self.version = PROGRAM_VERSION;
        self.last_update = LastUpdate::new(params.current_slot);
        self.lending_market = params.lending_market;
        self.liquidity = params.liquidity;
        self.collateral = params.collateral;
        self.config = params.config;
    }

    /// liquidityをdepositしてmintすべきcollateralの値を返す
    pub fn deposit_liquidity(&mut self, liquidity_amount: u64) -> Result<u64> {
        let collateral_amount = self
            .collateral_exchange_rate()?
            .liquidity_to_collateral(liquidity_amount)?;

        // liquidityをdeposit
        self.liquidity.deposit(liquidity_amount)?;
        // collateralをmint
        self.collateral.mint(collateral_amount)?;

        Ok(collateral_amount)
    }

    /// collateralをburnしてliquidityを返す
    pub fn redeem_collateral(&mut self, collateral_amount: u64) -> Result<u64> {
        let liquidity_amount = self
            .collateral_exchange_rate()?
            .collateral_to_liquidity(collateral_amount)?;

        // collateralをburn
        self.collateral.burn(collateral_amount)?;
        // liquidityをwithdraw
        self.liquidity.withdraw(liquidity_amount)?;

        Ok(liquidity_amount)
    }

    /// Collateral exchange rateを返す
    pub fn collateral_exchange_rate(&self) -> Result<CollateralExchangeRate> {
        let total_liquidity = self.liquidity.total_supply()?;
        self.collateral.exchange_rate(total_liquidity)
    }

    pub fn calculate_borrow(
        &self,
        amount_to_borrow: u64,
        max_borrow_value: u128,
    ) -> Result<CalculateBorrowResult> {
        let decimals = 10u64
            .checked_pow(self.liquidity.mint_decimals as u32)
            .ok_or(LendingError::MathOverflow)?;
        if amount_to_borrow == u64::MAX {
            let borrow_amount = max_borrow_value
                .checked_mul(decimals.into())
                .ok_or(LendingError::MathOverflow)?
                .checked_div(self.liquidity.market_price)
                .ok_or(LendingError::MathOverflow)?
                .min(self.liquidity.available_amount.into());

            let (borrow_fee, host_fee) = self
                .config
                .fees
                .calculate_borrow_fees(amount_to_borrow as u128, FeeCalculation::Inclusive)?;
            let receive_amount = borrow_amount
                .try_floor_u64()?
                .checked_sub(borrow_fee)
                .ok_or(LendingError::MathOverflow)?;

            Ok(CalculateBorrowResult {
                borrow_amount,
                receive_amount,
                borrow_fee,
                host_fee,
            })
        } else {
            let receive_amount = amount_to_borrow;
            let borrow_amount = receive_amount as u128;
            let (borrow_fee, host_fee) = self
                .config
                .fees
                .calculate_borrow_fees(borrow_amount, FeeCalculation::Exclusive)?;

            let borrow_amount = borrow_amount
                .checked_add(borrow_fee as u128)
                .ok_or(LendingError::MathOverflow)?;
            let borrow_value = borrow_amount
                .checked_mul(self.liquidity.market_price)
                .ok_or(LendingError::MathOverflow)?
                .checked_div(decimals as u128)
                .ok_or(LendingError::MathOverflow)?;
            require_gte!(max_borrow_value, borrow_value, LendingError::BorrowTooLarge);
            Ok(CalculateBorrowResult {
                borrow_amount,
                receive_amount,
                borrow_fee,
                host_fee,
            })
        }
    }

    pub fn calculate_repay(
        &self,
        amount_to_repay: u64,
        borrowed_amount: u128,
    ) -> Result<CalculateRepayResult> {
        let settle_amount = if amount_to_repay == u64::MAX {
            borrowed_amount
        } else {
            (amount_to_repay as u128).min(borrowed_amount)
        };
        let repay_amount = settle_amount.try_ceil_u64()?;

        Ok(CalculateRepayResult {
            settle_amount,
            repay_amount,
        })
    }
}
