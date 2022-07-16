use crate::{errors::LendingError, utils::byte_length::ByteLength};
use anchor_lang::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ObligationLiquidity {
    pub borrow_reserve: Pubkey,
    pub cumulative_borrow_rate_wads: u128,
    pub borrowed_amount_wads: u128,
    pub market_value: u128,
}

impl ByteLength for ObligationLiquidity {
    const LEN: usize = 32 + 16 + 16 + 16;
}

impl ObligationLiquidity {
    pub fn new(borrow_reserve: Pubkey) -> Self {
        Self {
            borrow_reserve,
            cumulative_borrow_rate_wads: 1,
            borrowed_amount_wads: 0,
            market_value: 0,
        }
    }

    pub fn repay(&mut self, settle_amount: u128) -> Result<()> {
        self.borrowed_amount_wads = self
            .borrowed_amount_wads
            .checked_sub(settle_amount)
            .ok_or(LendingError::MathOverflow)?;
        Ok(())
    }

    pub fn borrow(&mut self, borrow_amount: u128) -> Result<()> {
        self.borrowed_amount_wads = self
            .borrowed_amount_wads
            .checked_add(borrow_amount)
            .ok_or(LendingError::MathOverflow)?;
        Ok(())
    }
}
