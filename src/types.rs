use std::collections::HashMap;

#[cfg(feature = "character-sheet")]
use crate::character::{Attribute, Skill};
#[cfg(feature = "character-sheet")]
pub type AttributeMap = HashMap<String, Attribute>;
#[cfg(feature = "character-sheet")]
pub type SkillMap = HashMap<String, Skill>;

use crate::{
    locale::{LocaleTag, LocaleText},
    Data, UserData,
};

pub type UserId = u64;
pub type UsersHashMap = HashMap<UserId, UserData>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Data, Error>;

pub type LocaleVec = Vec<(LocaleTag, LocaleText)>;
pub type LocaleTextMap = HashMap<LocaleTag, LocaleText>;
pub type SkillValMap = Vec<(LocaleTag, i32)>;
