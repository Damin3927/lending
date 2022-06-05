use crate::{errors::LendingError, math::common::WAD, utils::byte_length::ByteLength};
use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct ReserveLiquidity {
    /// Reserve liquidity mint address
    pub mint_pubkey: Pubkey,

    /// Reserve liquidity mint decimals
    pub mint_decimals: u8,

    /// Reserve liquidity supply address
    pub supply_pubkey: Pubkey,

    /// Reserve liquidity fee receiver address
    pub fee_receiver: Pubkey,

    /// Reserve liquidity oracle account
    pub oracle_pubkey: Pubkey,

    /// Reserve liquidity available
    pub available_amount: u64,

    /// Reserve liquidity borrowed
    pub borrowed_amount_wads: u128,

    /// Reserve liquidity cumulative borrow rate
    pub cumulative_borrow_rate_wads: u128,

    /// Reserve liquidity market price in quote currency
    pub market_price: u128,
}

impl ByteLength for ReserveLiquidity {
    const LEN: usize = 32 + 1 + 32 + 32 + 32 + 8 + 16 + 16 + 16;
}

pub struct NewReserveLiquidityParams {
    /// Reserve liquidity mind address
    pub mint_pubkey: Pubkey,

    /// Reserve liquidity mint decimals
    pub mint_decimals: u8,

    /// Reserve liquidity supply address
    pub supply_pubkey: Pubkey,

    /// Reserve liquidity fee receiver address
    pub fee_receiver: Pubkey,

    /// Reserve liquidity oracle account
    pub oracle_pubkey: Pubkey,

    /// Reserve liquidity market price in quote currency
    pub market_price: u128,
}

impl ReserveLiquidity {
    pub fn new(params: NewReserveLiquidityParams) -> Self {
        Self {
            mint_pubkey: params.mint_pubkey,
            mint_decimals: params.mint_decimals,
            supply_pubkey: params.supply_pubkey,
            fee_receiver: params.fee_receiver,
            oracle_pubkey: params.oracle_pubkey,
            available_amount: 0,
            borrowed_amount_wads: 0,
            cumulative_borrow_rate_wads: WAD as u128,
            market_price: params.market_price,
        }
    }

    /// 借りられたtoken量込みの総供給可能量を返す
    pub fn total_supply(&self) -> Result<u128> {
        (self.available_amount as u128)
            .checked_add(self.borrowed_amount_wads)
            .ok_or(error!(LendingError::MathOverflow))
    }

    /// liquidityをdepositする
    pub fn deposit(&mut self, liquidity_amount: u64) -> Result<()> {
        self.available_amount = self
            .available_amount
            .checked_add(liquidity_amount)
            .ok_or(LendingError::MathOverflow)?;

        Ok(())
    }

    /// liquidityをwithdrawする
    pub fn withdraw(&mut self, liquidity_amount: u64) -> Result<()> {
        require_gte!(
            self.available_amount,
            liquidity_amount,
            LendingError::InsufficientLiquidity
        );

        self.available_amount = self
            .available_amount
            .checked_sub(liquidity_amount)
            .ok_or(LendingError::MathOverflow)?;

        Ok(())
    }
}
