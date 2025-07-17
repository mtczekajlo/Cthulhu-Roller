use crate::{
    Error,
    roller::{DiceResult, DiceRng, RollRegex},
};
use regex::Regex;

pub fn roll<D: DiceRng>(rng: &mut D, min: i32, max: i32) -> i32 {
    rng.random_range(min..=max)
}

pub fn roll_die<D: DiceRng>(rng: &mut D, sides: i32) -> i32 {
    roll(rng, 1, sides)
}

pub fn roll_dice<D: DiceRng>(
    rng: &mut D,
    dice_count: i32,
    dice_sides: i32,
    multiplier: f32,
    modifier: i32,
    sign: i32,
) -> DiceResult {
    let mut rolled: Vec<i32> = Vec::new();
    for _ in 0..dice_count {
        rolled.push(sign.signum() * roll_die(rng, dice_sides));
    }
    let sum = rolled.iter().sum::<i32>();
    let result: i32 = (sum as f32 * multiplier).ceil() as i32 + modifier;
    DiceResult::new(dice_count, dice_sides, result, rolled, modifier)
}

#[cfg(feature = "character-sheet")]
pub fn get_max(dice_count: i32, dice_sides: i32, multiplier: f32, modifier: i32, sign: i32) -> DiceResult {
    let mut rolled: Vec<i32> = Vec::new();
    for _ in 0..dice_count {
        rolled.push(sign.signum() * dice_sides);
    }
    let sum = rolled.iter().sum::<i32>();
    let result: i32 = (sum as f32 * multiplier).ceil() as i32 + modifier;
    DiceResult::new(dice_count, dice_sides, result, rolled, modifier)
}

pub fn roll_parse(input: &str) -> Result<Vec<RollRegex>, Error> {
    let pattern = r"(?P<dice>(?P<sign>[+-])?(?P<count>\d+)?[dk](?P<sides>\d+))?(?P<mult>[x*]([0-9]*[.])?[0-9]+)?(?P<mod>[+-]?\d+)?";
    let re = Regex::new(pattern)?;
    let dice_stripped = input.replace(' ', "");
    let tokens = split_inclusive(&dice_stripped, &['+', '-']);
    let mut results: Vec<RollRegex> = vec![];
    for token in tokens {
        let captures_v = re.captures_iter(token);
        for captures in captures_v {
            let sign = match captures.name("sign") {
                Some(m) => format!("{}1", m.as_str()).parse()?,
                None => 1,
            };
            let dice_sides = match captures.name("sides") {
                Some(m) => m.as_str().parse::<i32>()?,
                None => 0,
            };
            let dice_count = match captures.name("count") {
                Some(m) => m.as_str().parse()?,
                None => {
                    if dice_sides > 0 {
                        1
                    } else {
                        0
                    }
                }
            };
            let multiplier = match captures.name("mult") {
                Some(m) => m.as_str().replace('x', "").parse()?,
                None => 1.0,
            };
            let modifier = match captures.name("mod") {
                Some(m) => m.as_str().parse()?,
                None => 0,
            };
            let roll_regex = RollRegex::new(sign, dice_count, dice_sides, multiplier, modifier);
            results.push(roll_regex);
        }
    }
    Ok(results)
}

fn split_inclusive<'a>(input: &'a str, p: &[char]) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    let mut last = 0;
    for (i, _) in input.match_indices(p) {
        if last != i {
            tokens.push(&input[last..i]);
        }
        last = i;
    }
    tokens.push(&input[last..]);
    tokens
}

pub fn merge_dice_results(dice_results: &[DiceResult]) -> Result<DiceResult, Error> {
    let result = 0.max(dice_results.iter().map(|dr| dr.result).sum());
    let mut roll_msg = String::new();
    let rolls = dice_results
        .iter()
        .flat_map(|dr| {
            roll_msg.push_str(&dr.roll_msg);
            roll_msg.push('\n');
            dr.rolls.clone()
        })
        .collect();
    Ok(DiceResult {
        result,
        rolls,
        roll_msg,
    })
}

#[cfg(feature = "character-sheet")]
pub fn get_roll_max(input: &str) -> Result<DiceResult, Error> {
    let mut dice_results: Vec<DiceResult> = vec![];
    for roll_regex in roll_parse(input)? {
        dice_results.push(get_max(
            roll_regex.dice_count,
            roll_regex.dice_sides,
            roll_regex.multiplier,
            roll_regex.modifier,
            roll_regex.sign,
        ))
    }
    merge_dice_results(&dice_results)
}

pub fn roll_impl<D: DiceRng>(rng: &mut D, input: &str) -> Result<DiceResult, Error> {
    let mut dice_results: Vec<DiceResult> = vec![];
    for roll_regex in roll_parse(input)? {
        dice_results.push(roll_dice(
            rng,
            roll_regex.dice_count,
            roll_regex.dice_sides,
            roll_regex.multiplier,
            roll_regex.modifier,
            roll_regex.sign,
        ))
    }
    merge_dice_results(&dice_results)
}
