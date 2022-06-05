#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Rate(u128);

impl Rate {}

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
