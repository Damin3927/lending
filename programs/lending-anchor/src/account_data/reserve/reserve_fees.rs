use crate::utils::byte_length::ByteLength;
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
