pub mod obligation_collateral;
pub mod obligation_liquidity;

use self::{
    obligation_collateral::ObligationCollateral, obligation_liquidity::ObligationLiquidity,
};
use crate::{
    account_data::last_update::LastUpdate, constants::PROGRAM_VERSION,
    utils::byte_length::ByteLength,
};
use anchor_lang::prelude::*;

#[account]
pub struct Obligation {
    pub version: u8,
    pub last_update: LastUpdate,
    pub lending_market: Pubkey,
    pub owner: Pubkey,
    pub deposits: Vec<ObligationCollateral>,
    pub borrows: Vec<ObligationLiquidity>,
    pub deposited_value: u128,
    pub borrowed_value: u128,
    pub allowed_borrow_value: u128,
    pub unhealthy_borrow_value: u128,
}

impl ByteLength for Obligation {
    const LEN: usize = 0;
}

pub struct InitObligationParams {
    pub current_slot: u64,
    pub lending_market: Pubkey,
    pub owner: Pubkey,
    pub deposits: Vec<ObligationCollateral>,
    pub borrows: Vec<ObligationLiquidity>,
}

impl Obligation {
    pub fn init(&mut self, params: InitObligationParams) {
        self.version = PROGRAM_VERSION;
        self.last_update = LastUpdate::new(params.current_slot);
        self.lending_market = params.lending_market;
        self.owner = params.owner;
        self.deposits = params.deposits;
        self.borrows = params.borrows;
    }

    pub fn repay(&mut self, settle_amount: u128, liquidity_index: usize) -> Result<()> {
        let liquidity = &mut self.borrows[liquidity_index];
        if settle_amount == liquidity.borrowed_amount_wads {
            self.borrows.remove(liquidity_index);
        } else {
            liquidity.repay(settle_amount)?;
        }

        Ok(())
    }
}
