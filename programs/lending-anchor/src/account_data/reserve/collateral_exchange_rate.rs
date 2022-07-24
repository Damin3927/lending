use crate::{errors::LendingError, math::rate::Rate};

#[derive(Clone, Copy, Debug)]
pub struct CollateralExchangeRate(pub Rate);

/// collateral と liquidityの相互変換のためのstruct
impl CollateralExchangeRate {
    /// Reserve Collateral を liquidityに変換
    pub fn collateral_to_liquidity(&self, collateral_amount: u64) -> Result<u64, LendingError> {
        collateral_amount
            .checked_div(self.0.into())
            .ok_or(LendingError::InvalidConfig)
    }

    pub fn liquidity_to_collateral(&self, liquidity_amount: u64) -> Result<u64, LendingError> {
        liquidity_amount
            .checked_mul(self.0.into())
            .ok_or(LendingError::InvalidConfig)
    }
}

impl From<CollateralExchangeRate> for Rate {
    fn from(exchange_rate: CollateralExchangeRate) -> Self {
        exchange_rate.0
    }
}
