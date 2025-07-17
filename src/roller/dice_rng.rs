use rand::{prelude::*, rngs::ThreadRng};
use std::ops::RangeInclusive;

pub trait DiceRng {
    fn random_range(&mut self, range: RangeInclusive<i32>) -> i32;
}

pub struct RealRng {
    rng: ThreadRng,
}

impl RealRng {
    pub fn new() -> Self {
        Self { rng: rand::rng() }
    }
}

impl DiceRng for RealRng {
    fn random_range(&mut self, range: RangeInclusive<i32>) -> i32 {
        self.rng.random_range(range)
    }
}
