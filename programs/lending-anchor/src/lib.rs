pub mod account_data;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod utils;

use crate::instructions::{init_lending_market::*, set_lending_market_owner::*};
use anchor_lang::{prelude::*, solana_program::entrypoint::ProgramResult};

declare_id!("5svw6ndVHyYaUASP5njKKCMM8GiaXGTJZiycw7TGY5Y2");

#[program]
pub mod lending_anchor {
    use super::*;

    pub fn init_lending_market(
        ctx: Context<InitLendingMarket>,
        quote_currency: [u8; 32],
    ) -> ProgramResult {
        process_init_lending_market(ctx, quote_currency)
    }

    pub fn set_lending_market_owner(
        ctx: Context<SetLendingMarketOwner>,
        new_owner: Pubkey,
    ) -> ProgramResult {
        process_set_lending_market_owner(ctx, new_owner)
    }
}
