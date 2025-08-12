use crate::locale::LocaleTag;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ModifierDiceType {
    Bonus,
    Penalty,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
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
