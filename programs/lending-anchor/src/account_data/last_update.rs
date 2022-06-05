use crate::utils::byte_length::ByteLength;
use anchor_lang::{prelude::*, solana_program::clock::Slot};

#[derive(Copy, Clone, Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct LastUpdate {
    /// Last slot when updated
    pub slot: u64,
    /// True when marked stale, false when slot updated
    pub stale: bool,
}

impl ByteLength for LastUpdate {
    const LEN: usize = 8 + 1;
}

impl LastUpdate {
    pub fn new(slot: Slot) -> Self {
        Self { slot, stale: true }
    }
}
