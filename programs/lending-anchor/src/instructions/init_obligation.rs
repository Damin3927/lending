use crate::{
    account_data::{
        lending_market::LendingMarket,
        obligation::{InitObligationParams, Obligation},
    },
    errors::LendingError,
};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct InitObligation<'info> {
    pub obligation: Box<Account<'info, Obligation>>,
    pub lending_market: Box<Account<'info, LendingMarket>>,
    pub obligation_owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn process_init_obligation(ctx: Context<InitObligation>) -> Result<()> {
    require_keys_eq!(
        *ctx.accounts.obligation.to_account_info().owner,
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require_keys_eq!(
        *ctx.accounts.lending_market.to_account_info().owner,
        *ctx.program_id,
        LendingError::InvalidMarketOwner
    );
    require_keys_eq!(
        ctx.accounts.lending_market.token_program_id,
        *ctx.accounts.token_program.key,
        LendingError::InvalidTokenProgram
    );

    ctx.accounts.obligation.init(InitObligationParams {
        current_slot: Clock::get()?.slot,
        lending_market: ctx.accounts.lending_market.key(),
        owner: ctx.accounts.obligation_owner.key(),
        deposits: vec![],
        borrows: vec![],
    });

    Ok(())
}
