use crate::{
    account_data::reserve::collateral_exchange_rate::CollateralExchangeRate,
    constants::INITIAL_COLLATERAL_RATE, errors::LendingError, math::rate::Rate,
    utils::byte_length::ByteLength,
};
use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct ReserveCollateral {
    /// Reserve collateral mint address
    pub mint_pubkey: Pubkey,

    /// Reserve collateral mint supply, used for exchange rate
    pub mint_total_supply: u64,

    /// Reserve collateral supply address
    pub supply_pubkey: Pubkey,
}

impl ByteLength for ReserveCollateral {
    const LEN: usize = 32 + 8 + 32;
}

impl ReserveCollateral {
    pub fn new(mint_pubkey: Pubkey, mint_total_supply: u64, supply_pubkey: Pubkey) -> Self {
        Self {
            mint_pubkey,
            mint_total_supply,
            supply_pubkey,
        }
    }

    /// 現在のcollateral exchange rateを返す
    pub fn exchange_rate(&self, total_liquidity: u128) -> Result<CollateralExchangeRate> {
        let rate = if self.mint_total_supply == 0 || total_liquidity == 0 {
            Rate::try_from(INITIAL_COLLATERAL_RATE)
        } else {
            let mint_total_supply = self.mint_total_supply;
            Rate::try_from(
                mint_total_supply
                    .checked_div(total_liquidity as u64)
                    .ok_or(error!(LendingError::MathOverflow))?,
            )
        }
        .map_err(|_| LendingError::MathOverflow)?;

        Ok(CollateralExchangeRate(rate))
    }

    /// collateral をmintする
    /// total supplyに加える
    pub fn mint(&mut self, collateral_amount: u64) -> Result<()> {
        self.mint_total_supply = self
            .mint_total_supply
            .checked_add(collateral_amount)
            .ok_or(LendingError::MathOverflow)?;

        Ok(())
    }

    /// collateral をburnする
    /// total supplyから引く
    pub fn burn(&mut self, collateral_amount: u64) -> Result<()> {
        self.mint_total_supply = self
            .mint_total_supply
            .checked_sub(collateral_amount)
            .ok_or(LendingError::MathOverflow)?;
        Ok(())
    }
}
