use super::common::PERCENT_SCALER;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Rate(u128);

impl Rate {
    pub fn from_percent(percent: u8) -> Self {
        Self((percent as u64 * PERCENT_SCALER) as u128)
    }

    pub fn zero() -> Self {
        Self(0)
    }
}

impl From<u64> for Rate {
    fn from(rate: u64) -> Self {
        Rate(rate as u128)
    }
}

impl Into<u64> for Rate {
    fn into(self) -> u64 {
        self.0 as u64
    }
}
