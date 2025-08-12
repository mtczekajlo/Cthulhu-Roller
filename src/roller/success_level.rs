use crate::{
    locale::{LocaleLang, LocaleTag, locale_entry_by_tag, locale_tag_by_str},
    types::Error,
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum SuccessLevel {
    CriticalFailure,
    Failure,
    Success,
    HardSuccess,
    ExtremeSuccess,
    CriticalSuccess,
}

impl SuccessLevel {
    pub fn rank(&self) -> u8 {
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
            Self::HardSuccess => 1.max(threshold / 2),
            Self::ExtremeSuccess => 1.max(threshold / 5),
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

    pub fn to_string_lang(self, lang: LocaleLang) -> String {
        locale_entry_by_tag(self.to_locale_tag()).get(lang)
    }

    pub fn from_tag(tag: LocaleTag) -> Result<Self, Error> {
        match tag {
            LocaleTag::Success => Ok(Self::Success),
            LocaleTag::HardSuccess => Ok(Self::HardSuccess),
            LocaleTag::ExtremeSuccess => Ok(Self::ExtremeSuccess),
            _ => Err("SuccessLevel from tag error".into()),
        }
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

impl Display for SuccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&locale_entry_by_tag(self.to_locale_tag()).get(LocaleLang::default()))
    }
}

impl FromStr for SuccessLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag = locale_tag_by_str(s).ok_or(Error::from("error"))?;

        match tag {
            LocaleTag::CriticalFailure => Ok(SuccessLevel::CriticalFailure),
            LocaleTag::Failure => Ok(SuccessLevel::Failure),
            LocaleTag::Success => Ok(SuccessLevel::Success),
            LocaleTag::HardSuccess => Ok(SuccessLevel::HardSuccess),
            LocaleTag::ExtremeSuccess => Ok(SuccessLevel::ExtremeSuccess),
            LocaleTag::CriticalSuccess => Ok(SuccessLevel::CriticalSuccess),
            _ => Err("error".into()),
        }
    }
}
