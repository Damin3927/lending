use crate::{account_data::lending_market::*, utils::byte_length::ByteLength};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct InitLendingMarket<'info> {
    /// Signer of this instruction
    /// will be the owner field of lending_market
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = LendingMarket::LEN + 8,
    )]
    pub lending_market: Account<'info, LendingMarket>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    /// CHECK:
    pub oracle: UncheckedAccount<'info>,
}

pub fn process_init_lending_market(
    ctx: Context<InitLendingMarket>,
    quote_currency: [u8; 32],
) -> Result<()> {
    let lending_market = &ctx.accounts.lending_market.key();
    ctx.accounts.lending_market.init(InitLendingMarketParams {
        bump_seed: Pubkey::find_program_address(&[lending_market.as_ref()], ctx.program_id).1,
        owner: ctx.accounts.owner.key(),
        quote_currency,
        token_program_id: ctx.accounts.token_program.key(),
        oracle_program_id: ctx.accounts.oracle.key(),
    });
    Ok(())
}
