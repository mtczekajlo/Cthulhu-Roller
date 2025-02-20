use std::{
    cmp::{Ordering, Reverse},
    fmt::Display,
};

use rand::prelude::*;

pub enum SuccessRate {
    CriticalFailure,
    Failure,
    Success,
    HardSuccess,
    ExtremeSuccess,
    CriticalSuccess,
}

impl Display for SuccessRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CriticalFailure => "CRITICAL FAILURE!",
            Self::Failure => "Failure",
            Self::Success => "Success",
            Self::HardSuccess => "Hard Success! (½)",
            Self::ExtremeSuccess => "Extreme Success! (⅕)",
            Self::CriticalSuccess => "CRITICAL SUCCESS!",
        })
    }
}

impl SuccessRate {
    pub fn hex(&self) -> u32 {
        match &self {
            Self::CriticalFailure => 0xBE29EC,
            Self::Failure => 0x800080,
            Self::Success => 0x415D43,
            Self::HardSuccess => 0x709775,
            SuccessRate::ExtremeSuccess => 0x8FB996,
            Self::CriticalSuccess => 0xB3CBB9,
        }
    }
}

pub struct RollSkillResult {
    pub result: i32,
    pub one_roll: i32,
    pub ten_rolls: Vec<i32>,
    pub threshold: i32,
    pub penalty: i32,
    pub bonus: i32,
    pub success_rate: Option<SuccessRate>,
}

pub struct RollDiceResult {
    pub result: i32,
    pub rolls: Vec<i32>,
    pub modifier: Option<i32>,
    pub multiplier: Option<i32>,
}

fn roll(min: i32, max: i32) -> i32 {
    rand::rng().random_range(min..(max + 1))
}

pub fn roll_die(sides: i32) -> i32 {
    roll(1, sides)
}

pub fn roll_dice(
    dice_count: i32,
    sides: i32,
    modifier: Option<i32>,
    multiplier: Option<i32>,
) -> RollDiceResult {
    let mut rolled: Vec<i32> = Vec::new();
    for _ in 0..dice_count {
        rolled.push(roll_die(sides));
    }
    let sum = rolled.iter().sum::<i32>();
    let result: i32 = sum * multiplier.unwrap_or(1) + modifier.unwrap_or(0);
    RollDiceResult {
        result,
        rolls: rolled,
        modifier,
        multiplier,
    }
}

fn reduce_dice(penalty_dice: i32, bonus_dice: i32) -> (i32, i32) {
    match penalty_dice.cmp(&bonus_dice) {
        Ordering::Greater => (penalty_dice - bonus_dice, 0),
        Ordering::Equal => (0, 0),
        Ordering::Less => (0, bonus_dice - penalty_dice),
    }
}

pub fn roll_skill(threshold: i32, penalty_dice: i32, bonus_dice: i32) -> RollSkillResult {
    let (penalty, bonus) = reduce_dice(penalty_dice, bonus_dice);
    let one_result = roll(0, 9);
    let mut ten_results = vec![roll(0, 9)];
    let mut ten_result = ten_results[0];

    if penalty > 0 {
        for _ in 0..penalty {
            ten_results.push(roll(0, 9));
        }
        ten_result = *ten_results.iter().max().unwrap();
    } else if bonus > 0 {
        for _ in 0..bonus {
            ten_results.push(roll(0, 9));
        }
        ten_result = *ten_results.iter().min().unwrap();
    }

    let result: i32;
    let success_rate: Option<SuccessRate>;

    match (one_result, ten_result) {
        (0, 0) => {
            result = 100;
            success_rate = Some(SuccessRate::CriticalFailure);
        }
        (1, 0) => {
            result = 1;
            success_rate = Some(SuccessRate::CriticalSuccess);
        }
        _ => {
            result = ten_result * 10 + one_result;
            if threshold < 50 && result >= 96 {
                success_rate = Some(SuccessRate::CriticalFailure);
            } else if result <= threshold / 5 {
                success_rate = Some(SuccessRate::ExtremeSuccess);
            } else if result <= threshold / 2 {
                success_rate = Some(SuccessRate::HardSuccess);
            } else if result <= threshold {
                success_rate = Some(SuccessRate::Success);
            } else {
                success_rate = Some(SuccessRate::Failure)
            }
        }
    }

    ten_results.sort_by_key(|el| Reverse(*el));

    RollSkillResult {
        result,
        one_roll: one_result,
        ten_rolls: ten_results,
        threshold,
        penalty,
        bonus,
        success_rate,
    }
}
