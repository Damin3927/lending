use crate::{account_data::lending_market::*, errors::LendingError::*};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetLendingMarketOwner<'info> {
    #[account(mut, has_one = owner @ InvalidMarketOwner)]
    pub lending_market: Account<'info, LendingMarket>,
    pub owner: Signer<'info>,
}

pub fn process_set_lending_market_owner(
    ctx: Context<SetLendingMarketOwner>,
    new_owner: Pubkey,
) -> Result<()> {
    ctx.accounts.lending_market.owner = new_owner;
    Ok(())
}
