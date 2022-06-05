pub mod account_data;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod math;
pub mod pyth;
pub mod utils;

use crate::{
    account_data::reserve::reserve_config::ReserveConfig,
    instructions::{init_lending_market::*, init_reserve::*, set_lending_market_owner::*},
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
}
