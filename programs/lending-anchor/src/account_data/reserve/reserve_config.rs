use crate::{account_data::reserve::reserve_fees::ReserveFees, utils::byte_length::ByteLength};
use anchor_lang::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct ReserveConfig {
    /// Optimal utilization rate, as a percentage
    pub optimal_utilization_rate: u8,

    /// Target ratio of the value of borrows to deposits, as a percentage
    /// 0 if uses as collateral is disabled
    pub loan_to_value_ratio: u8,

    /// Bonus a liquidator gets when repaying part of an unhealthy obligation, as a percentage
    pub liquidation_bonus: u8,

    /// Loan to value ratio at which an obligation can be liquidated, as a percentage
    pub liquidation_threshold: u8,

    /// Min borrow APY
    pub min_borrow_rate: u8,

    /// Optimal (uitilization) borrow APY
    pub optimal_borrow_rate: u8,

    /// Max borrow APY
    pub max_borrow_rate: u8,

    /// Program owner fees assessed, separate from gains due to interest accrual
    pub fees: ReserveFees,
}

impl ByteLength for ReserveConfig {
    const LEN: usize = 1 + 1 + 1 + 1 + 1 + 1 + 1 + ReserveFees::LEN;
}
