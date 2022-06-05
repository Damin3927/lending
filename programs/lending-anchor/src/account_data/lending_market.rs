use crate::{constants::PROGRAM_VERSION, utils::byte_length::ByteLength};
use anchor_lang::prelude::*;

#[account]
pub struct LendingMarket {
    /// Version of lending market
    pub version: u8,

    /// Bump seed for derived authority address
    pub bump_seed: u8,

    /// Owner authority which can add new reserves
    pub owner: Pubkey,

    /// Currency market prices are quoted in
    pub quote_currency: [u8; 32],

    pub token_program_id: Pubkey,
    pub oracle_program_id: Pubkey,
}

impl ByteLength for LendingMarket {
    const LEN: usize = 1 + 1 + 32 + 32 + 32 + 32;
}

pub struct InitLendingMarketParams {
    pub bump_seed: u8,
    pub owner: Pubkey,
    pub quote_currency: [u8; 32],
    pub token_program_id: Pubkey,
    pub oracle_program_id: Pubkey,
}

impl LendingMarket {
    /// Create a new lending market
    pub fn init(&mut self, params: InitLendingMarketParams) {
        self.version = PROGRAM_VERSION;
        self.bump_seed = params.bump_seed;
        self.owner = params.owner;
        self.quote_currency = params.quote_currency;
        self.token_program_id = params.token_program_id;
        self.oracle_program_id = params.oracle_program_id;
    }
}
