use crate::roller::attribute_roll::AttributeRollResult;
use crate::utils::split_inclusive;
use crate::{
    Error,
    roller::dice_rng::{DiceRng, RealRng},
};
use itertools::Itertools;
use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct RollRegex {
    pub query: String,
    pub dice_count: i32,
    pub dice_sides: i32,
    pub multiplier: f32,
    pub modifier: i32,
}

impl RollRegex {
    pub fn new(dice_count: i32, dice_sides: i32, multiplier: f32, modifier: i32) -> Self {
        let query = format!(
            "{}{}",
            if dice_count != 0 {
                format!("{}d{}", multiplier.signum() as i32 * dice_count, dice_sides)
            } else {
                "".into()
            },
            if modifier != 0 {
                format!("{modifier:+}")
            } else {
                "".into()
            }
        );
        Self {
            query,
            dice_count,
            dice_sides,
            multiplier,
            modifier,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RollResult {
    pub query: String,
    result: i32,
    pub rolls: Vec<i32>,
    pub modifier: i32,
    pub roll_msg: String,
}

impl RollResult {
    pub fn new(query: &str, result: i32, rolls: Vec<i32>, modifier: i32) -> Self {
        let mut roll_msg = String::new();
        roll_msg.push_str(
            rolls
                .iter()
                .map(|r| format!("`[{r}]`"))
                .collect::<Vec<_>>()
                .join(" ")
                .as_str(),
        );
        if modifier != 0 {
            roll_msg.push_str(format!("`{modifier:+}`").as_str());
        }
        Self {
            query: query.into(),
            result,
            rolls,
            modifier,
            roll_msg,
        }
    }

    #[cfg(feature = "character-sheet")]
    fn new_modifier(modifier: i32) -> RollResult {
        Self::new(format!("{modifier:+}").as_str(), modifier, vec![], modifier)
    }

    pub fn result_real(&self) -> i32 {
        self.result
    }

    pub fn result(&self) -> i32 {
        0.max(self.result)
    }
}

pub fn roll_range<D: DiceRng>(rng: &mut D, min: i32, max: i32) -> i32 {
    rng.random_range(min..=max)
}

pub fn roll_die<D: DiceRng>(rng: &mut D, sides: i32) -> i32 {
    roll_range(rng, 1, sides)
}

pub fn roll_dice<D: DiceRng>(
    rng: &mut D,
    query: &str,
    dice_count: i32,
    dice_sides: i32,
    multiplier: f32,
    modifier: i32,
) -> RollResult {
    let mut rolled: Vec<i32> = Vec::new();
    for _ in 0..dice_count {
        rolled.push(multiplier.signum() as i32 * roll_die(rng, dice_sides));
    }
    let sum = rolled.iter().sum::<i32>();
    let result: i32 = (sum as f32 * multiplier.abs()).ceil() as i32 + modifier;
    RollResult::new(query, result, rolled, modifier)
}

pub fn roll_parse(query: &str) -> Result<Vec<RollRegex>, Error> {
    let pattern = r"(?P<dice>(?P<sign>[+-])?(?P<count>\d+)?[dk](?P<sides>\d+))?(?P<mult>[x*]([0-9]*[.])?[0-9]+)?(?P<mod>[+-]?\d+)?";
    let re = Regex::new(pattern)?;
    let dice_stripped = query.replace(' ', "");
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
                Some(m) => m.as_str().replace(['x', '*'], "").parse()?,
                None => 1.0,
            } * sign as f32;
            let modifier = match captures.name("mod") {
                Some(m) => m.as_str().parse()?,
                None => 0,
            };
            let roll_regex = RollRegex::new(dice_count, dice_sides, multiplier, modifier);
            if roll_regex.query.is_empty() {
                continue;
            }
            results.push(roll_regex);
        }
    }
    Ok(results)
}

pub fn roll_dice_no_query<D: DiceRng>(
    rng: &mut D,
    dice_count: i32,
    dice_sides: i32,
    multiplier: f32,
    modifier: i32,
) -> RollResult {
    roll_dice(rng, "", dice_count, dice_sides, multiplier, modifier)
}

pub fn merge_roll_results(roll_results: &[RollResult]) -> Result<RollResult, Error> {
    let query = roll_results
        .iter()
        .map(|dr| dr.query.clone())
        .fold(String::new(), |s, q| {
            format!("{}{}", s, if q.starts_with(['+', '-']) { q } else { format!("+{q}") })
        });
    let result = roll_results.iter().map(|dr| dr.result).sum();
    let roll_msg = roll_results.iter().map(|dr| &dr.roll_msg).join(" ");
    let rolls = roll_results.iter().flat_map(|dr| dr.rolls.clone()).collect();
    Ok(RollResult {
        query,
        result,
        rolls,
        modifier: 0,
        roll_msg,
    })
}

#[cfg(feature = "character-sheet")]
pub fn rollregex_to_max(roll_regex: &RollRegex) -> RollRegex {
    RollRegex::new(
        0,
        0,
        0.0,
        (roll_regex.multiplier * roll_regex.dice_count as f32 * roll_regex.dice_sides as f32) as i32
            + roll_regex.modifier,
    )
}

#[cfg(feature = "character-sheet")]
pub fn get_roll_max(input: &str) -> Result<RollResult, Error> {
    let mut dice_results: Vec<RollResult> = vec![];
    for roll_regex in roll_parse(input)?.iter().map(rollregex_to_max) {
        dice_results.push(RollResult::new_modifier(roll_regex.modifier));
    }
    merge_roll_results(&dice_results)
}

pub fn roll_impl<D: DiceRng>(rng: &mut D, query: &str) -> Result<RollResult, Error> {
    let mut roll_results: Vec<RollResult> = vec![];
    for roll_regex in roll_parse(query)? {
        let roll_result = roll_dice(
            rng,
            &roll_regex.query,
            roll_regex.dice_count,
            roll_regex.dice_sides,
            roll_regex.multiplier,
            roll_regex.modifier,
        );
        roll_results.push(roll_result)
    }
    merge_roll_results(&roll_results)
}

pub fn roll_attributes(pulp_core_attribute: Option<&str>) -> AttributeRollResult {
    let mut rng = RealRng::new();
    AttributeRollResult::new(&mut rng, pulp_core_attribute)
}
