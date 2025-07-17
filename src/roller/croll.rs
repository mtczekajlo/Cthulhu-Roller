use crate::{
    Error,
    roller::{
        ImproveResult, ModifierDice, ModifierDiceType, RealRng, SkillResult,
        roll::{roll, roll_die},
    },
};
use std::cmp::Ordering;

fn reduce_modifier_dice(penalty_dice: i32, bonus_dice: i32) -> Option<ModifierDice> {
    match penalty_dice.cmp(&bonus_dice) {
        Ordering::Greater => Some(ModifierDice::new(ModifierDiceType::Penalty, penalty_dice - bonus_dice)),
        Ordering::Equal => None,
        Ordering::Less => Some(ModifierDice::new(ModifierDiceType::Bonus, bonus_dice - penalty_dice)),
    }
}

pub fn roll_skill(threshold: i32, penalty_dice: i32, bonus_dice: i32) -> Result<SkillResult, Error> {
    let mut rng = RealRng::new();
    let one_result = roll(&mut rng, 0, 9);
    let mut ten_result = roll(&mut rng, 0, 9);
    let mut ten_results = vec![ten_result];
    let modifier_dice = reduce_modifier_dice(penalty_dice, bonus_dice);

    if let Some(modifier_dice) = &modifier_dice {
        for _ in 0..modifier_dice.count {
            ten_results.push(roll(&mut rng, 0, 9));
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

    Ok(SkillResult::new(
        threshold,
        result,
        one_result,
        ten_results,
        modifier_dice,
    ))
}

pub fn improve_skill(threshold: i32) -> ImproveResult {
    let mut rng = RealRng::new();
    ImproveResult::new(threshold, roll_die(&mut rng, 100))
}
