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

    #[account(
        constraint = lending_market.token_program_id == token_program.key() @ LendingError::InvalidTokenProgram,
    )]
    pub lending_market: Box<Account<'info, LendingMarket>>,

    pub obligation_owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn process_init_obligation(ctx: Context<InitObligation>) -> Result<()> {
    ctx.accounts.obligation.init(InitObligationParams {
        current_slot: Clock::get()?.slot,
        lending_market: ctx.accounts.lending_market.key(),
        owner: ctx.accounts.obligation_owner.key(),
        deposits: vec![],
        borrows: vec![],
    });

    Ok(())
}
