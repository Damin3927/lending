use crate::{
    errors::LendingError,
    math::{
        common::{Decimal, WAD},
        rate::Rate,
    },
    utils::byte_length::ByteLength,
};
use anchor_lang::prelude::*;

/// Additional fee information on a reserve
#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ReserveFees {
    pub borrow_fee_wad: u64,
    pub flash_loan_fee_wad: u64,
    pub host_fee_percentage: u8,
}

impl ByteLength for ReserveFees {
    const LEN: usize = 8 + 8 + 1;
}

pub enum FeeCalculation {
    Exclusive,
    Inclusive,
}

impl ReserveFees {
    pub fn calculate_borrow_fees(
        &self,
        borrow_amount: u128,
        fee_calculation: FeeCalculation,
    ) -> Result<(u64, u64)> {
        self.calculate_fees(borrow_amount, self.borrow_fee_wad, fee_calculation)
    }

    fn calculate_fees(
        &self,
        amount: u128,
        fee_wad: u64,
        fee_calculation: FeeCalculation,
    ) -> Result<(u64, u64)> {
        let borrow_fee_rate = Rate::from_scaled_val(fee_wad);
        let borrow_fee_rate_uint: u64 = borrow_fee_rate.into();
        let host_fee_rate = Rate::from_percent(self.host_fee_percentage);
        if borrow_fee_rate > Rate::zero() && amount > 0 {
            let need_to_assess_host_fee = host_fee_rate > Rate::zero();
            let minimum_fee = if need_to_assess_host_fee {
                2u64 // 1 token to owner, 1 to host
            } else {
                1u64 // 1 token to owner, nothing else
            };

            let borrow_fee_amount = match fee_calculation {
                // Calculate fee to be added to borrow: fee = amount * rate
                FeeCalculation::Exclusive => amount
                    .checked_mul(borrow_fee_rate_uint as u128)
                    .ok_or(LendingError::MathOverflow)?,
                // Calculate fee to be subtracted from borrow: fee = amount * (rate / (rate + 1))
                FeeCalculation::Inclusive => {
                    let borrow_fee_rate = borrow_fee_rate_uint
                        .checked_div(borrow_fee_rate_uint + WAD)
                        .ok_or(LendingError::MathOverflow)?;
                    amount
                        .checked_mul(borrow_fee_rate as u128)
                        .ok_or(LendingError::MathOverflow)?
                }
            };

            let borrow_fee_decimal = borrow_fee_amount.max(minimum_fee.into());
            if borrow_fee_decimal >= amount {
                msg!("Borrow amount is too small to receive liquidity after fees");
                return Err(LendingError::BorrowTooSmall.into());
            }

            let borrow_fee = borrow_fee_decimal.try_round_u64()?;
            let host_fee = if need_to_assess_host_fee {
                let host_fee_rate: u64 = host_fee_rate.into();
                borrow_fee_decimal
                    .checked_mul(host_fee_rate as u128)
                    .ok_or(LendingError::MathOverflow)?
                    .try_round_u64()?
                    .max(1u64)
            } else {
                0
            };

            Ok((borrow_fee, host_fee))
        } else {
            Ok((0, 0))
        }
    }
}
