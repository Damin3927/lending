use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

pub mod account_data;
pub mod constants;
pub mod instructions;
pub mod utils;

use crate::instructions::init_lending_market::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod lending_anchor {
    use super::*;

    pub fn init_lending_market(
        ctx: Context<InitLendingMarket>,
        owner: Pubkey,
        quote_currency: [u8; 32],
    ) -> ProgramResult {
        msg!("Instruction: Init Lending Market");
        process_init_lending_market(ctx, owner, quote_currency)
    }
}
