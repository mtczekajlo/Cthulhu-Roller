use crate::locale::LocaleTag;
use std::cmp::Ordering;
pub mod croll;
pub use croll::*;
pub mod dice_rng;
pub use dice_rng::*;
pub mod roll;
pub use roll::*;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SuccessLevel {
    CriticalFailure,
    Failure,
    Success,
    HardSuccess,
    ExtremeSuccess,
    CriticalSuccess,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModifierDiceType {
    Bonus,
    Penalty,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModifierDice {
    pub dice_type: ModifierDiceType,
    pub count: i32,
}

impl ModifierDiceType {
    pub fn to_locale_tag(&self) -> LocaleTag {
        match self {
            ModifierDiceType::Bonus => LocaleTag::Bonus,
            ModifierDiceType::Penalty => LocaleTag::Penalty,
        }
    }
}

impl ModifierDice {
    pub fn new(dice_type: ModifierDiceType, count: i32) -> Self {
        Self { dice_type, count }
    }
}

impl SuccessLevel {
    fn rank(&self) -> u8 {
        match self {
            SuccessLevel::CriticalFailure => 0,
            SuccessLevel::Failure => 1,
            SuccessLevel::Success => 2,
            SuccessLevel::HardSuccess => 3,
            SuccessLevel::ExtremeSuccess => 4,
            SuccessLevel::CriticalSuccess => 5,
        }
    }

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

    pub fn threshold(&self, threshold: i32) -> i32 {
        match self {
            Self::CriticalFailure => {
                if threshold < 50 {
                    96
                } else {
                    100
                }
            }
            Self::Failure => threshold - 1,
            Self::Success => threshold,
            Self::HardSuccess => threshold / 2,
            Self::ExtremeSuccess => threshold / 5,
            Self::CriticalSuccess => 100,
        }
    }

    pub fn to_locale_tag(self) -> LocaleTag {
        match self {
            SuccessLevel::CriticalFailure => LocaleTag::CriticalFailure,
            SuccessLevel::Failure => LocaleTag::Failure,
            SuccessLevel::Success => LocaleTag::Success,
            SuccessLevel::HardSuccess => LocaleTag::HardSuccess,
            SuccessLevel::ExtremeSuccess => LocaleTag::ExtremeSuccess,
            SuccessLevel::CriticalSuccess => LocaleTag::CriticalSuccess,
        }
    }

    pub fn delta(&self, result: i32, threshold: i32) -> i32 {
        result - self.threshold(threshold)
    }
}

impl Ord for SuccessLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for SuccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Iterator for SuccessLevel {
    type Item = SuccessLevel;
    fn next(&mut self) -> Option<Self::Item> {
        let next = match self {
            SuccessLevel::Failure => Some(SuccessLevel::Success),
            SuccessLevel::Success => Some(SuccessLevel::HardSuccess),
            SuccessLevel::HardSuccess => Some(SuccessLevel::ExtremeSuccess),
            _ => return None,
        };
        *self = next.unwrap();
        next
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SkillResult {
    pub success_level: SuccessLevel,
    pub result: i32,
    pub one_roll: i32,
    pub ten_rolls: Vec<i32>,
    pub threshold: i32,
    pub modifier_dice: Option<ModifierDice>,
}

impl SkillResult {
    fn new(
        threshold: i32,
        result: i32,
        one_roll: i32,
        ten_rolls: Vec<i32>,
        modifier_dice: Option<ModifierDice>,
    ) -> Self {
        let success_level = match result {
            100 => SuccessLevel::CriticalFailure,
            1 => SuccessLevel::CriticalSuccess,
            _ => {
                if threshold < 50 && result >= 96 {
                    SuccessLevel::CriticalFailure
                } else if result <= threshold / 5 {
                    SuccessLevel::ExtremeSuccess
                } else if result <= threshold / 2 {
                    SuccessLevel::HardSuccess
                } else if result <= threshold {
                    SuccessLevel::Success
                } else {
                    SuccessLevel::Failure
                }
            }
        };

        Self {
            threshold,
            result,
            one_roll,
            ten_rolls,
            modifier_dice,
            success_level,
        }
    }
}

impl PartialOrd for SkillResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SkillResult {
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

#[derive(Debug, Clone)]
pub struct DiceResult {
    pub result: i32,
    pub rolls: Vec<i32>,
    pub roll_msg: String,
}
impl DiceResult {
    fn new(dice_count: i32, dice_sides: i32, result: i32, rolls: Vec<i32>, modifier: i32) -> Self {
        let mut roll_msg = String::new();
        if dice_sides != 0 {
            roll_msg.push_str(
                rolls
                    .iter()
                    .fold(format!("{}d{}:", dice_count, dice_sides), |s, r| {
                        format!("{} `[{}]`", s, r)
                    })
                    .as_str(),
            );
        }
        if modifier != 0 {
            roll_msg.push_str(format!("`{:+}`", modifier).as_str());
        }
        Self {
            result,
            rolls,
            roll_msg,
        }
    }
}

#[derive(Clone)]
pub struct ImproveResult {
    pub result: i32,
    pub success_level: SuccessLevel,
    pub threshold: i32,
}

impl ImproveResult {
    pub fn new(threshold: i32, result: i32) -> Self {
        let success_level = match result > threshold || result > 95 {
            true => SuccessLevel::Success,
            _ => SuccessLevel::Failure,
        };
        Self {
            threshold,
            result,
            success_level,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CharacterInitiative {
    pub result: SkillResult,
    pub name: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq)]
pub struct RollRegex {
    pub sign: i32,
    pub dice_count: i32,
    pub dice_sides: i32,
    pub multiplier: f32,
    pub modifier: i32,
}

impl RollRegex {
    pub fn new(sign: i32, dice_count: i32, dice_sides: i32, multiplier: f32, modifier: i32) -> Self {
        Self {
            sign,
            dice_count,
            dice_sides,
            multiplier,
            modifier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{predicate::*, *};
    use rstest::rstest;

    #[automock]
    trait MockableDiceRng {
        fn random_range(&mut self, range: std::ops::RangeInclusive<i32>) -> i32;
    }

    impl DiceRng for MockMockableDiceRng {
        fn random_range(&mut self, range: std::ops::RangeInclusive<i32>) -> i32 {
            MockableDiceRng::random_range(self, range)
        }
    }

    #[rstest]
    #[case("2", vec![RollRegex::new(1,0,0,1.0,2)])]
    #[case("+2", vec![RollRegex::new(1,0,0,1.0,2)])]
    #[case("-2", vec![RollRegex::new(1,0,0,1.0,-2)])]
    #[case("k10", vec![RollRegex::new(1,1,10,1.0,0)])]
    #[case("1k10", vec![RollRegex::new(1,1,10,1.0,0)])]
    #[case("k10x3", vec![RollRegex::new(1,1,10,3.0,0)])]
    #[case("k10x0.5", vec![RollRegex::new(1,1,10,0.5,0)])]
    #[case("k10+2", vec![RollRegex::new(1,1,10,1.0,0),RollRegex::new(1,0,0,1.0,2)])]
    #[case("k10x3+2", vec![RollRegex::new(1,1,10,3.0,0),RollRegex::new(1,0,0,1.0,2)])]
    #[case("k10+k10", vec![RollRegex::new(1,1,10,1.0,0),RollRegex::new(1,1,10,1.0,0)])]
    #[case("k10-k10", vec![RollRegex::new(1,1,10,1.0,0),RollRegex::new(-1,1,10,1.0,0)])]
    #[case("1k10+1k10", vec![RollRegex::new(1,1,10,1.0,0),RollRegex::new(1,1,10,1.0,0)])]
    #[case("1k10-1k10", vec![RollRegex::new(1,1,10,1.0,0),RollRegex::new(-1,1,10,1.0,0)])]
    #[case("2k10+2k10", vec![RollRegex::new(1,2,10,1.0,0),RollRegex::new(1,2,10,1.0,0)])]
    #[case("2k10-2k10", vec![RollRegex::new(1,2,10,1.0,0),RollRegex::new(-1,2,10,1.0,0)])]
    fn test_roll_parse(#[case] input: &str, #[case] expected: Vec<RollRegex>) {
        let result = roll_parse(input);
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case("2", 2)]
    #[case("+2", 2)]
    #[case("-2", 0)]
    #[case("k10", 5)]
    #[case("1k10", 5)]
    #[case("k10+2", 5+2)]
    #[case("k10x3", 5*3)]
    #[case("k10x0.5", 3)]
    #[case("k10x3-2", 5*3-2)]
    #[case("k10-2", 5-2)]
    #[case("k10x3-2", 5*3-2)]
    #[case("1k10", 5)]
    #[case("1k10+2", 5+2)]
    #[case("1k10x3+2", 5*3+2)]
    #[case("1k10-2", 5-2)]
    #[case("1k10x3-2", 5*3-2)]
    #[case("k10+k10", 5+5)]
    #[case("1k10+k10", 5+5)]
    #[case("k10+1k10", 5+5)]
    #[case("1k10+1k10", 5+5)]
    #[case("k10+2k10", 5+2*5)]
    #[case("1k10+2k10", 5+2*5)]
    #[case("k10+2k10", 5+2*5)]
    #[case("1k10+2k10", 5+2*5)]
    #[case("k10+2+k10+2", 5+2+5+2)]
    #[case("k10x3+2+k10x3+2", 5*3+2+5*3+2)]
    #[case("k10-2+k10-2", 5-2+5-2)]
    #[case("k10x3-2+k10x3-2", 5*3-2+5*3-2)]
    #[case("k10-k10", 0)]
    #[case("k10+2-k10+2", 5+2-5+2)]
    #[case("k10x3+2-k10x3+2", 5*3+2-5*3+2)]
    #[case("k10-2-k10-2", 0)]
    #[case("k10x3-2-k10x3-2", 0)]
    fn test_roll_impl(#[case] input: &str, #[case] expected: i32) {
        let mut mock_rng = MockMockableDiceRng::new();
        mock_rng.expect_random_range().returning(|_| 5);
        let dr = roll_impl(&mut mock_rng, input);
        let dr = dr.unwrap();
        assert_eq!(dr.result, expected);
    }

    #[cfg(feature = "character-sheet")]
    #[rstest]
    #[case("2", 2)]
    #[case("+2", 2)]
    #[case("-2", 0)]
    #[case("k4", 4)]
    #[case("k10", 10)]
    #[case("k10+k6", 16)]
    #[case("k3+k4", 7)]
    #[case("1k4", 4)]
    #[case("1k10", 10)]
    #[case("1k10+1k6", 16)]
    #[case("1k3+1k4", 7)]
    #[case("k10-k6", 4)]
    #[case("k3-k4", 0)]
    #[case("1k10-1k6", 4)]
    #[case("1k3-1k4", 0)]
    #[case("1k3+2", 5)]
    #[case("1k3-2", 1)]
    fn test_roll_max(#[case] input: &str, #[case] expected: i32) {
        let dr = get_roll_max(input);
        let dr = dr.unwrap();
        assert_eq!(dr.result, expected);
    }
}
