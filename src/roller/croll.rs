use serde::{Deserialize, Serialize};

use crate::{
    Error,
    roller::{
        dice_rng::RealRng,
        modifier_dice::{ModifierDice, ModifierDiceType},
        roll::roll_range,
        success_level::SuccessLevel,
    },
};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CrollResult {
    pub query: String,
    pub success_level: SuccessLevel,
    result: i32,
    pub one_roll: i32,
    pub ten_rolls: Vec<i32>,
    pub threshold: i32,
    pub modifier_dice: Option<ModifierDice>,
}

impl CrollResult {
    fn new(
        query: &str,
        threshold: i32,
        result: i32,
        one_roll: i32,
        ten_rolls: Vec<i32>,
        modifier_dice: Option<ModifierDice>,
    ) -> Self {
        let mut slf = Self {
            query: query.into(),
            threshold,
            result,
            one_roll,
            ten_rolls,
            modifier_dice,
            success_level: SuccessLevel::Failure,
        };
        slf.update_success_level();
        slf
    }

    fn update_success_level(&mut self) {
        self.success_level = match self.result {
            100 => SuccessLevel::CriticalFailure,
            1 => SuccessLevel::CriticalSuccess,
            _ => {
                if self.threshold < 50 && self.result >= 96 {
                    SuccessLevel::CriticalFailure
                } else if self.result <= self.threshold / 5 {
                    SuccessLevel::ExtremeSuccess
                } else if self.result <= self.threshold / 2 {
                    SuccessLevel::HardSuccess
                } else if self.result <= self.threshold {
                    SuccessLevel::Success
                } else {
                    SuccessLevel::Failure
                }
            }
        };
    }

    pub fn result(&self) -> i32 {
        self.result
    }

    pub fn set_result(&mut self, result: i32) {
        self.result = result;
        self.update_success_level();
    }
}

impl PartialOrd for CrollResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CrollResult {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.success_level.rank().cmp(&other.success_level.rank()) {
            Ordering::Equal => match self.threshold.cmp(&other.threshold) {
                Ordering::Equal => self.result.cmp(&other.result),
                other => other.reverse(),
            },
            other => other.reverse(),
        }
    }
}

fn reduce_modifier_dice(penalty_dice: i32, bonus_dice: i32) -> Option<ModifierDice> {
    match penalty_dice.cmp(&bonus_dice) {
        Ordering::Greater => Some(ModifierDice::new(ModifierDiceType::Penalty, penalty_dice - bonus_dice)),
        Ordering::Equal => None,
        Ordering::Less => Some(ModifierDice::new(ModifierDiceType::Bonus, bonus_dice - penalty_dice)),
    }
}

pub fn croll(query: &str, threshold: i32, penalty_dice: i32, bonus_dice: i32) -> Result<CrollResult, Error> {
    let mut rng = RealRng::new();
    let one_result = roll_range(&mut rng, 0, 9);
    let mut ten_result = roll_range(&mut rng, 0, 9);
    let mut ten_results = vec![ten_result];
    let modifier_dice = reduce_modifier_dice(penalty_dice, bonus_dice);

    if let Some(modifier_dice) = &modifier_dice {
        for _ in 0..modifier_dice.count {
            ten_results.push(roll_range(&mut rng, 0, 9));
        }

        if one_result == 0 {
            let ten_results: Vec<i32> = ten_results.iter().map(|&el| if el == 0 { 10 } else { el }).collect();
            ten_result = match modifier_dice.dice_type {
                ModifierDiceType::Bonus => *ten_results.iter().min().ok_or("Min not found")?,
                ModifierDiceType::Penalty => *ten_results.iter().max().ok_or("Max not found")?,
            };
        } else {
            ten_result = match modifier_dice.dice_type {
                ModifierDiceType::Bonus => *ten_results.iter().min().ok_or("Min not found")?,
                ModifierDiceType::Penalty => *ten_results.iter().max().ok_or("Max not found")?,
            };
        }
    }

    let result = match (one_result, ten_result) {
        (0, 0) => 100,
        (1, 0) => 1,
        _ => ten_result * 10 + one_result,
    };

    Ok(CrollResult::new(
        query,
        threshold,
        result,
        one_result,
        ten_results,
        modifier_dice,
    ))
}
