use std::{
    cmp::{Ordering, Reverse},
    fmt::Display,
};

use rand::prelude::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
pub enum SuccessLevel {
    CriticalFailure,
    Failure,
    Success,
    HardSuccess,
    ExtremeSuccess,
    CriticalSuccess,
}

impl Display for SuccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CriticalFailure => "🐙 CRITICAL FAILURE! 🐙",
            Self::Failure => "❌ Failure",
            Self::Success => "⭐ Success",
            Self::HardSuccess => "⭐⭐ Hard Success! (½)",
            Self::ExtremeSuccess => "⭐⭐⭐ Extreme Success! (⅕)",
            Self::CriticalSuccess => "✨ CRITICAL SUCCESS! ✨",
        })
    }
}

impl SuccessLevel {
    pub fn hex(&self) -> u32 {
        match &self {
            Self::CriticalFailure => 0xBE29EC,
            Self::Failure => 0x800080,
            Self::Success => 0x415D43,
            Self::HardSuccess => 0x709775,
            Self::ExtremeSuccess => 0x8FB996,
            Self::CriticalSuccess => 0xB3CBB9,
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct SkillResult {
    pub success_level: SuccessLevel,
    pub result: i32,
    pub one_roll: i32,
    pub ten_rolls: Vec<i32>,
    pub threshold: i32,
    pub penalty: i32,
    pub bonus: i32,
}

impl PartialOrd for SkillResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SkillResult {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.success_level.cmp(&other.success_level) {
            Ordering::Equal => match self.threshold.cmp(&other.threshold) {
                Ordering::Equal => self.result.cmp(&other.result),
                other => other.reverse(),
            },
            other => other.reverse(),
        }
    }
}

#[derive(Clone)]
pub struct DiceResult {
    pub result: i32,
    pub rolls: Vec<i32>,
    pub modifier: Option<i32>,
    pub multiplier: Option<i32>,
}

#[derive(Clone)]
pub struct ImproveResult {
    pub result: i32,
    pub success_level: SuccessLevel,
    pub threshold: i32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CharacterInitiative {
    pub result: SkillResult,
    pub name: String,
}

#[derive(Clone)]
pub struct InitiativeResult {
    pub characters: Vec<CharacterInitiative>,
}

impl InitiativeResult {
    pub fn new(characters: Vec<CharacterInitiative>) -> Self {
        let mut ir = InitiativeResult { characters };
        ir.characters.sort();
        ir
    }
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
) -> DiceResult {
    let mut rolled: Vec<i32> = Vec::new();
    for _ in 0..dice_count {
        rolled.push(roll_die(sides));
    }
    let sum = rolled.iter().sum::<i32>();
    let result: i32 = sum * multiplier.unwrap_or(1) + modifier.unwrap_or(0);
    DiceResult {
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

pub fn roll_skill(threshold: i32, penalty_dice: i32, bonus_dice: i32) -> SkillResult {
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
    let success_level: SuccessLevel;

    match (one_result, ten_result) {
        (0, 0) => {
            result = 100;
            success_level = SuccessLevel::CriticalFailure;
        }
        (1, 0) => {
            result = 1;
            success_level = SuccessLevel::CriticalSuccess;
        }
        _ => {
            result = ten_result * 10 + one_result;
            if threshold < 50 && result >= 96 {
                success_level = SuccessLevel::CriticalFailure;
            } else if result <= threshold / 5 {
                success_level = SuccessLevel::ExtremeSuccess;
            } else if result <= threshold / 2 {
                success_level = SuccessLevel::HardSuccess;
            } else if result <= threshold {
                success_level = SuccessLevel::Success;
            } else {
                success_level = SuccessLevel::Failure
            }
        }
    }

    ten_results.sort_by_key(|el| Reverse(*el));

    SkillResult {
        result,
        one_roll: one_result,
        ten_rolls: ten_results,
        threshold,
        penalty,
        bonus,
        success_level,
    }
}

pub fn improve_skill(threshold: i32) -> ImproveResult {
    let result = roll(1, 100);
    let success_level = match result > threshold || result > 95 {
        true => SuccessLevel::Success,
        _ => SuccessLevel::Failure,
    };
    ImproveResult {
        result,
        success_level,
        threshold,
    }
}
