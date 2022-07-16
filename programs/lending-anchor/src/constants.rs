use anchor_lang::prelude::*;

use crate::math::common::WAD;

#[constant]
pub const PROGRAM_VERSION: u8 = 1;

/// collateral と liquidityの比率
/// もし5ならば，colalteralはliquidityの5倍価値が薄いということ
#[constant]
pub const INITIAL_COLLATERAL_RATIO: u64 = 1;
#[constant]
pub const INITIAL_COLLATERAL_RATE: u64 = INITIAL_COLLATERAL_RATIO * WAD;

#[constant]
pub const MAX_OBLIGATION_RESERVE: usize = 10;
