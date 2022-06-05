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
            reserve_liquidity::ReserveLiquidity,
        },
    },
    constants::PROGRAM_VERSION,
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
            .collateral_to_liquidity(liquidity_amount)?;

        // liquidityをdeposit
        self.liquidity.deposit(liquidity_amount)?;
        // collateralをmint
        self.collateral.mint(collateral_amount)?;

        Ok(collateral_amount)
    }

    /// Collateral exchange rateを返す
    pub fn collateral_exchange_rate(&self) -> Result<CollateralExchangeRate> {
        let total_liquidity = self.liquidity.total_supply()?;
        self.collateral.exchange_rate(total_liquidity)
    }
}
