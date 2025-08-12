use std::collections::HashMap;

#[cfg(feature = "character-sheet")]
use crate::character::{Attribute, Skill};
#[cfg(feature = "character-sheet")]
pub type AttributeMap = HashMap<String, Attribute>;
#[cfg(feature = "character-sheet")]
pub type SkillMap = HashMap<String, Skill>;

use crate::bot_data::{ContextData, UserData};
use crate::locale::{LocaleEntry, LocaleTag};

pub type UserId = u64;
pub type UsersHashMap = HashMap<UserId, UserData>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, ContextData, Error>;
#[cfg(feature = "character-sheet")]
pub type ApplicationContext<'a> = poise::ApplicationContext<'a, ContextData, Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, ContextData, Error>;

pub type LocaleVec = Vec<(LocaleTag, LocaleEntry)>;
pub type LocaleEntryMap = HashMap<LocaleTag, LocaleEntry>;
#[cfg(feature = "character-sheet")]
pub type SkillValMap = Vec<(LocaleTag, i32)>;
