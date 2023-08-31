use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ticks(u32);

impl Ticks {
    pub fn new(ticks: u32) -> Self {
        Ticks(ticks)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl Add for Ticks {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Ticks(self.0 + other.0)
    }
}

impl Sub for Ticks {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Ticks(self.0 - other.0)
    }
}
