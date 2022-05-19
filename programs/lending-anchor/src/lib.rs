pub mod account_data;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod utils;

use crate::instructions::init_lending_market::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

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
}
