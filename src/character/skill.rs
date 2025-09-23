use crate::{
    locale::{LocaleEntry, LocaleLang, LocaleTag},
    types::SkillValMap,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Skill {
    pub name: LocaleEntry,
    pub value: i32,
    pub to_improve: bool,
    pub improvable: bool,
    pub default: bool,
    pub constant: bool,
}

impl Skill {
    pub fn new_default(text: LocaleEntry, value: i32) -> Self {
        Self {
            name: text,
            value,
            improvable: true,
            default: true,
            ..Default::default()
        }
    }

    pub fn new_default_unimprovable(text: LocaleEntry, value: i32) -> Self {
        Self {
            name: text,
            value,
            default: true,
            ..Default::default()
        }
    }

    pub fn new_default_constant(text: LocaleEntry, value: i32) -> Self {
        Self {
            name: text,
            value,
            default: true,
            constant: true,
            ..Default::default()
        }
    }

    pub fn new_custom(text: LocaleEntry, value: i32) -> Self {
        Self {
            name: text,
            value,
            improvable: true,
            ..Default::default()
        }
    }

    pub fn set(&mut self, value: i32) -> Result<(), SkillError> {
        if self.constant {
            return Err(SkillError::Constant(self.name.clone()));
        }
        self.value = value;
        Ok(())
    }

    pub fn modify(&mut self, value: i32) -> Result<(), SkillError> {
        self.set(self.value + value)
    }
}

pub enum SkillError {
    AlreadyExists(String),
    CantRemove(String),
    Constant(LocaleEntry),
    NoSuchSkill(String),
}

impl SkillError {
    pub fn to_string(&self, lang: LocaleLang) -> String {
        match (self, lang) {
            (SkillError::AlreadyExists(n), LocaleLang::Polski) => format!("Umiejętność `{n}` już istnieje."),
            (SkillError::AlreadyExists(n), LocaleLang::English) => format!("Skill `{n}` already exists."),
            (SkillError::CantRemove(n), LocaleLang::Polski) => {
                format!("Nie można usunąć podstawowej umiejętności `{n}`.")
            }
            (SkillError::CantRemove(n), LocaleLang::English) => format!("Can't remove basic skill `{n}`."),
            (SkillError::Constant(n), LocaleLang::Polski) => {
                format!("Umiejętności `{}` nie można modyfikować.", n.pl)
            }
            (SkillError::Constant(n), LocaleLang::English) => format!("Skill `{}` can't be modified.", n.en),
            (SkillError::NoSuchSkill(n), LocaleLang::Polski) => format!("Nie ma takiej umiejętności `{n}`."),
            (SkillError::NoSuchSkill(n), LocaleLang::English) => format!("No such skill `{n}`"),
        }
    }
}

lazy_static! {
    pub static ref DEFAULT_SKILLS: SkillValMap = vec![
        (LocaleTag::Accounting, 5),
        (LocaleTag::Anthropology, 1),
        (LocaleTag::Appraise, 5),
        (LocaleTag::Archeology, 1),
        (LocaleTag::Charm, 15),
        (LocaleTag::Climb, 20),
        (LocaleTag::Disguise, 5),
        (LocaleTag::Dodge, 0),
        (LocaleTag::DriveAuto, 20),
        (LocaleTag::ElectricalRepair, 10),
        (LocaleTag::Electronics, 1),
        (LocaleTag::FastTalk, 5),
        (LocaleTag::FightingBrawl, 25),
        (LocaleTag::FirearmsHandgun, 20),
        (LocaleTag::FirearmsRifleShotgun, 25),
        (LocaleTag::FirstAid, 30),
        (LocaleTag::History, 5),
        (LocaleTag::Intimidate, 15),
        (LocaleTag::Jump, 20),
        (LocaleTag::LanguageOwn, 0),
        (LocaleTag::Law, 5),
        (LocaleTag::LibraryUse, 20),
        (LocaleTag::Listen, 20),
        (LocaleTag::Locksmith, 1),
        (LocaleTag::MechanicalRepair, 10),
        (LocaleTag::Medicine, 1),
        (LocaleTag::NaturalWorld, 10),
        (LocaleTag::Navigate, 10),
        (LocaleTag::Occult, 5),
        (LocaleTag::OperateHeavyMachinery, 1),
        (LocaleTag::Persuade, 10),
        (LocaleTag::Psychoanalysis, 1),
        (LocaleTag::Psychology, 10),
        (LocaleTag::Ride, 5),
        (LocaleTag::SleightOfHand, 10),
        (LocaleTag::SpotHidden, 25),
        (LocaleTag::Stealth, 20),
        (LocaleTag::Swim, 10),
        (LocaleTag::Throw, 10),
        (LocaleTag::Track, 10),
    ]
    .into_iter()
    .collect();
    pub static ref ADDITIONAL_SKILLS: SkillValMap = vec![
        (LocaleTag::AnimalHandling, 1),
        (LocaleTag::Artillery, 1),
        (LocaleTag::ComputerUse, 1),
        (LocaleTag::Demolitions, 1),
        (LocaleTag::Diving, 1),
        (LocaleTag::Hypnosis, 1),
        (LocaleTag::ReadLips, 1),
    ]
    .into_iter()
    .collect();
    pub static ref SPECIALIZED_SKILLS: SkillValMap = vec![
        (LocaleTag::ArtCraft, 5),
        (LocaleTag::Fighting, 10),
        (LocaleTag::Firearms, 10),
        (LocaleTag::LanguageOther, 1),
        (LocaleTag::Lore, 1),
        (LocaleTag::Pilot, 1),
        (LocaleTag::Science, 1),
        (LocaleTag::Survival, 10),
    ]
    .into_iter()
    .collect();
    pub static ref DEFAULT_UNIMPROVABLE_SKILLS: SkillValMap =
        vec![(LocaleTag::CreditRating, 0), (LocaleTag::CthulhuMythos, 0),]
            .into_iter()
            .collect();
    pub static ref DEFAULT_CONSTANT_SKILLS: SkillValMap = vec![
        (LocaleTag::ArtCraftAny, 5),
        (LocaleTag::LanguageAnyOther, 1),
        (LocaleTag::PilotAny, 1),
        (LocaleTag::ScienceAny, 1),
        (LocaleTag::SurvivalAny, 10),
    ]
    .into_iter()
    .collect();
}
