use crate::errors::LendingError;
use anchor_lang::prelude::*;

/// Identity (?)
pub const WAD: u64 = 1_000_000_000_000_000_000;

pub const PERCENT_SCALER: u64 = 10_000_000_000_000_000;

pub trait Decimal {
    fn try_floor_u64(&self) -> Result<u64>;
    fn try_ceil_u64(&self) -> Result<u64>;
    fn try_round_u64(&self) -> Result<u64>;
}

impl Decimal for u128 {
    fn try_floor_u64(&self) -> Result<u64> {
        Ok(u64::try_from(
            self.checked_div(WAD as u128)
                .ok_or(LendingError::MathOverflow)?,
        )
        .map_err(|_| LendingError::MathOverflow)?)
    }

    fn try_ceil_u64(&self) -> Result<u64> {
        Ok(u64::try_from(
            (WAD as u128)
                .checked_sub(1)
                .ok_or(LendingError::MathOverflow)?
                .checked_add(*self)
                .ok_or(LendingError::MathOverflow)?,
        )
        .map_err(|_| LendingError::MathOverflow)?)
    }

    fn try_round_u64(&self) -> Result<u64> {
        Ok(u64::try_from(
            self.checked_add(*self)
                .ok_or(LendingError::MathOverflow)?
                .checked_div(WAD as u128)
                .ok_or(LendingError::MathOverflow)?,
        )
        .map_err(|_| LendingError::MathOverflow)?)
    }
}
