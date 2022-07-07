use crate::{utils::byte_length::ByteLength, errors::LendingError};
use anchor_lang::{prelude::*, solana_program::clock::Slot};

/// Number of slots to consider stale after
pub const STALE_AFTER_SLOTS_ELAPSED: u64 = 1;

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

    pub fn slots_elapsed(&self, slot: u64) -> Result<u64> {
        let slots_elapsed = slot.checked_sub(self.slot).ok_or(LendingError::MathOverflow)?;
        Ok(slots_elapsed)
    }

    pub fn update_slot(&mut self, slot: u64) {
        self.slot = slot;
        self.stale = false;
    }

    pub fn mark_stale(&mut self) {
        self.stale= true;
    }

    /// Check if marked stale or last update slot is too long ago
    pub fn is_stale(&self, slot: u64) -> Result<bool> {
        Ok(self.stale || self.slots_elapsed(slot)? >= STALE_AFTER_SLOTS_ELAPSED)
    }
}
