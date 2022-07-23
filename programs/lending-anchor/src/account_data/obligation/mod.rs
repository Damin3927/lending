pub mod obligation_collateral;
pub mod obligation_liquidity;

use self::{
    obligation_collateral::ObligationCollateral, obligation_liquidity::ObligationLiquidity,
};
use crate::{
    account_data::last_update::LastUpdate,
    constants::{MAX_OBLIGATION_RESERVE, PROGRAM_VERSION},
    errors::LendingError,
    math::rate::Rate,
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

    pub fn withdraw(&mut self, withdraw_amount: u64, collateral_index: usize) -> Result<()> {
        let collateral = &mut self.deposits[collateral_index];
        if withdraw_amount == collateral.deposited_amount {
            self.deposits.remove(collateral_index);
        } else {
            collateral.withdraw(withdraw_amount)?;
        }
        Ok(())
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

    pub fn find_or_add_collateral_to_deposits(
        &mut self,
        deposit_reserve: Pubkey,
    ) -> Result<&mut ObligationCollateral> {
        if let Some(collateral_index) = self._find_collateral_index_in_deposits(deposit_reserve) {
            return Ok(&mut self.deposits[collateral_index]);
        }

        if self.deposits.len() + self.borrows.len() >= MAX_OBLIGATION_RESERVE {
            msg!(
                "Obligation cannot have more than {} deposits and borrows combined",
                MAX_OBLIGATION_RESERVE
            );
            return Err(LendingError::ObligationReserveLimit.into());
        }

        let collateral = ObligationCollateral::new(deposit_reserve);
        self.deposits.push(collateral);
        Ok(self.deposits.last_mut().unwrap())
    }

    fn _find_collateral_index_in_deposits(&self, deposit_reserve: Pubkey) -> Option<usize> {
        self.deposits
            .iter()
            .position(|collateral| collateral.deposit_reserve == deposit_reserve)
    }

    pub fn find_collateral_index_in_deposits(
        &self,
        deposit_reserve: Pubkey,
    ) -> Result<(&ObligationCollateral, usize)> {
        if self.deposits.is_empty() {
            msg!("Obligation has no deposits");
            return Err(LendingError::ObligatinoDepositsEmpty.into());
        }
        let collateral_index = self
            ._find_collateral_index_in_deposits(deposit_reserve)
            .ok_or(LendingError::InvalidObligationCollateral)?;
        Ok((&self.deposits[collateral_index], collateral_index))
    }

    pub fn find_or_add_liquidity_to_borrows(
        &mut self,
        borrow_reserve: Pubkey,
    ) -> Result<&mut ObligationLiquidity> {
        if let Some(liquidity_index) = self._find_liquidity_index_in_borrows(borrow_reserve) {
            return Ok(&mut self.borrows[liquidity_index]);
        }
        require_gt!(
            MAX_OBLIGATION_RESERVE,
            self.deposits.len() + self.borrows.len(),
            LendingError::ObligationReserveLimit
        );
        let liquidity = ObligationLiquidity::new(borrow_reserve);
        self.borrows.push(liquidity);
        Ok(self.borrows.last_mut().unwrap())
    }

    pub fn _find_liquidity_index_in_borrows(&self, borrow_reserve: Pubkey) -> Option<usize> {
        self.borrows
            .iter()
            .position(|liquidity| liquidity.borrow_reserve == borrow_reserve)
    }

    /// Calculate the maximum collateral value that can be withdrawn
    pub fn max_withdraw_value(&self, withdraw_collateral_ltv: Rate) -> Result<u128> {
        if self.allowed_borrow_value <= self.borrowed_value {
            return Ok(0);
        }
        if withdraw_collateral_ltv == Rate::zero() {
            return Ok(self.deposited_value);
        }
        let ltv: u64 = withdraw_collateral_ltv.into();
        self.allowed_borrow_value
            .checked_sub(self.borrowed_value)
            .ok_or(LendingError::MathOverflow)?
            .checked_div(ltv as u128)
            .ok_or(LendingError::MathOverflow.into())
    }

    pub fn remaining_borrow_value(&self) -> Result<u128> {
        self.allowed_borrow_value
            .checked_sub(self.borrowed_value)
            .ok_or(LendingError::MathOverflow.into())
    }
}
