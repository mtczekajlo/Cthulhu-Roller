use std::iter::zip;

use crate::{locale::LocaleLang, types::Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Weapon {
    pub name: String,
    pub range_dmgs: Vec<RangeDamage>,
    pub attacks: Vec<Attack>,
    pub attacks_query: String,
    pub apply_damage_modifier: bool,
    pub half_damage_modifier: bool,
    pub impaling: bool,
    pub skill: String,
    pub malfunction: Option<i32>,
    pub ammo: Option<Ammo>,
    pub default: bool,
}

pub enum WeaponOk {
    AddedRounds(i32),
    CriticalHit,
    Hit,
    Loaded(i32),
    Miss,
    RemovedRounds(i32),
    Shot(i32),
}

impl WeaponOk {
    pub fn to_string(&self, lang: LocaleLang) -> String {
        match (self, lang) {
            (WeaponOk::AddedRounds(r), LocaleLang::Polski) => format!("Dodano pocisków: {r}"),
            (WeaponOk::AddedRounds(r), LocaleLang::English) => format!("Added rounds: {r}"),
            (WeaponOk::CriticalHit, LocaleLang::Polski) => "Trafienie krytyczne!".into(),
            (WeaponOk::CriticalHit, LocaleLang::English) => "Critical Hit!".into(),
            (WeaponOk::Hit, LocaleLang::Polski) => "Trafienie!".into(),
            (WeaponOk::Hit, LocaleLang::English) => "Hit!".into(),
            (WeaponOk::Loaded(r), LocaleLang::Polski) => format!("Załadowano pocisków: {r}"),
            (WeaponOk::Loaded(r), LocaleLang::English) => format!("Loaded rounds: {r}"),
            (WeaponOk::Miss, LocaleLang::Polski) => "Pudło!".into(),
            (WeaponOk::Miss, LocaleLang::English) => "Miss!".into(),
            (WeaponOk::RemovedRounds(r), LocaleLang::Polski) => format!("Odjęto pocisków: {r}"),
            (WeaponOk::RemovedRounds(r), LocaleLang::English) => format!("Removed rounds: {r}"),
            (WeaponOk::Shot(r), LocaleLang::Polski) => format!("Wystrzelono pocisków: {r}"),
            (WeaponOk::Shot(r), LocaleLang::English) => format!("Shot rounds: {r}"),
        }
    }
}

pub enum WeaponError {
    DoesntUseAmmo,
    CantRemoveRounds(i32),
    CantShootRounds(i32),
    ClipEmpty,
    NoAmmo,
    NoNeedToReload,
}

impl WeaponError {
    pub fn to_string(&self, lang: LocaleLang) -> String {
        match (self, lang) {
            (WeaponError::DoesntUseAmmo, LocaleLang::Polski) => "Ta broń nie korzysta z amunicji.".into(),
            (WeaponError::DoesntUseAmmo, LocaleLang::English) => "This weapon doesn't use ammo.".into(),
            (WeaponError::CantRemoveRounds(rounds), LocaleLang::Polski) => {
                format!("Nie można usunąć {rounds} pocisków.")
            }
            (WeaponError::CantRemoveRounds(rounds), LocaleLang::English) => format!("Can't remove {rounds} rounds."),
            (WeaponError::ClipEmpty, LocaleLang::Polski) => "Pusty magazynek!".into(),
            (WeaponError::ClipEmpty, LocaleLang::English) => "Empty clip!".into(),
            (WeaponError::NoAmmo, LocaleLang::Polski) => "Brak amunicji!".into(),
            (WeaponError::NoAmmo, LocaleLang::English) => "No ammo left!".into(),
            (WeaponError::NoNeedToReload, LocaleLang::Polski) => "Nie ma potrzeby przeładowania.".into(),
            (WeaponError::NoNeedToReload, LocaleLang::English) => "No need to reload.".into(),
            (WeaponError::CantShootRounds(rounds), LocaleLang::Polski) => {
                format!("Nie można wystrzelić {rounds} pocisków")
            }
            (WeaponError::CantShootRounds(rounds), LocaleLang::English) => format!("Can't shoot {rounds} rounds"),
        }
    }
}

impl Weapon {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        range_dmgs: Vec<RangeDamage>,
        attacks_query: &str,
        apply_damage_modifier: bool,
        impaling: bool,
        skill: &str,
        malfunction: Option<i32>,
        ammo: Option<Ammo>,
    ) -> Self {
        Self {
            name: name.into(),
            range_dmgs,
            attacks_query: attacks_query.into(),
            attacks: attacks_from(attacks_query).unwrap(),
            apply_damage_modifier,
            half_damage_modifier: false,
            impaling,
            skill: skill.to_string(),
            malfunction,
            ammo,
            default: false,
        }
    }

    pub fn use_ammo(&mut self, rounds: i32) -> Result<WeaponOk, WeaponError> {
        if let Some(weapon) = self.ammo.as_mut() {
            Ok(weapon.shoot_rounds(rounds)?)
        } else {
            Err(WeaponError::DoesntUseAmmo)
        }
    }

    pub fn add_ammo(&mut self, rounds: i32) -> Result<WeaponOk, WeaponError> {
        if let Some(weapon) = self.ammo.as_mut() {
            Ok(weapon.add_rounds(rounds))
        } else {
            Err(WeaponError::DoesntUseAmmo)
        }
    }

    pub fn remove_ammo(&mut self, rounds: i32) -> Result<WeaponOk, WeaponError> {
        if let Some(weapon) = self.ammo.as_mut() {
            Ok(weapon.remove_rounds(rounds)?)
        } else {
            Err(WeaponError::DoesntUseAmmo)
        }
    }

    pub fn damage_dice(&self, lang: LocaleLang, distance: Option<i32>) -> String {
        let mut dmg = match distance {
            Some(m) => {
                self.range_dmgs
                    .iter()
                    .rev()
                    .find(|w| w.range.unwrap_or(i32::MAX) <= m)
                    .cloned()
                    .unwrap_or(self.range_dmgs[0].clone())
                    .damage
            }
            None => self.range_dmgs[0].damage.clone(),
        };
        if lang == LocaleLang::Polski {
            dmg = dmg.replace("d", "k");
        }
        dmg
    }

    pub fn apply_half_dm(&mut self) {
        self.half_damage_modifier = true;
    }

    pub fn reload(&mut self) -> Result<WeaponOk, WeaponError> {
        if let Some(weapon) = self.ammo.as_mut() {
            Ok(weapon.reload()?)
        } else {
            Err(WeaponError::DoesntUseAmmo)
        }
    }
}

impl PartialOrd for Weapon {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Weapon {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Weapon {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Weapon {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ammo {
    pub clip_rounds: i32,
    pub loose_rounds: i32,
    pub clip_size: i32,
}

impl Ammo {
    pub fn new(clip_size: i32, rounds: i32) -> Self {
        Self {
            clip_size,
            loose_rounds: rounds,
            clip_rounds: 0,
        }
    }

    pub fn reload(&mut self) -> Result<WeaponOk, WeaponError> {
        let mut rounds_to_load = self.clip_size - self.clip_rounds;

        if rounds_to_load == 0 {
            return Err(WeaponError::NoNeedToReload);
        } else if self.loose_rounds == 0 {
            return Err(WeaponError::NoAmmo);
        } else if self.loose_rounds < rounds_to_load {
            rounds_to_load = self.loose_rounds;
        }

        self.clip_rounds += rounds_to_load;
        self.loose_rounds -= rounds_to_load;

        Ok(WeaponOk::Loaded(rounds_to_load))
    }

    pub fn shoot_rounds(&mut self, rounds_to_shoot: i32) -> Result<WeaponOk, WeaponError> {
        if self.clip_rounds == 0 {
            return Err(WeaponError::ClipEmpty);
        }

        if self.clip_rounds < rounds_to_shoot {
            return Err(WeaponError::CantShootRounds(rounds_to_shoot));
        }

        self.clip_rounds -= rounds_to_shoot;

        Ok(WeaponOk::Shot(rounds_to_shoot))
    }

    pub fn add_rounds(&mut self, rounds_to_add: i32) -> WeaponOk {
        self.loose_rounds += rounds_to_add;
        WeaponOk::AddedRounds(rounds_to_add)
    }

    pub fn remove_rounds(&mut self, rounds_to_remove: i32) -> Result<WeaponOk, WeaponError> {
        if rounds_to_remove > self.loose_rounds + self.clip_rounds {
            return Err(WeaponError::CantRemoveRounds(rounds_to_remove));
        }

        if rounds_to_remove > self.loose_rounds {
            self.clip_rounds -= rounds_to_remove - self.loose_rounds;
            self.loose_rounds = 0;
        } else {
            self.loose_rounds -= rounds_to_remove;
        }

        Ok(WeaponOk::RemovedRounds(rounds_to_remove))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AttacksModifier {
    None,
    OnePenaltyToAll,
    IncreasingPenalty,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attack {
    pub count: i32,
    pub modifier: AttacksModifier,
}

impl Attack {
    pub fn new(count: i32, modifier: AttacksModifier) -> Self {
        Self { count, modifier }
    }

    pub fn get_modifier(&self, attack_number: i32) -> String {
        match self.modifier {
            AttacksModifier::None => "".into(),
            AttacksModifier::OnePenaltyToAll => "-".into(),
            AttacksModifier::IncreasingPenalty => "-".repeat(attack_number as usize),
        }
    }
}

pub fn attacks_from(value: &str) -> Result<Vec<Attack>, Error> {
    let value = value.replace(' ', "");
    let value = value.as_str();
    match value {
        "1" => Ok(vec![Attack::new(1, AttacksModifier::None)]),
        "2" => Ok(vec![
            Attack::new(1, AttacksModifier::None),
            Attack::new(2, AttacksModifier::None),
        ]),
        "1(2)" => Ok(vec![
            Attack::new(1, AttacksModifier::None),
            Attack::new(2, AttacksModifier::OnePenaltyToAll),
        ]),
        "1(3)" => Ok(vec![
            Attack::new(1, AttacksModifier::None),
            Attack::new(3, AttacksModifier::OnePenaltyToAll),
        ]),
        "1or2" | "1lub2" => Ok(vec![
            Attack::new(1, AttacksModifier::None),
            Attack::new(2, AttacksModifier::None),
        ]),
        _ => Err("Couldn't parse into Attack obj".into()),
    }
}

#[derive(Debug)]
pub enum RangeDamageError {
    RangeDamageDiffers,
}

impl RangeDamageError {
    pub fn to_string(&self, lang: LocaleLang) -> String {
        match (self, lang) {
            (RangeDamageError::RangeDamageDiffers, LocaleLang::Polski) => {
                "Ilości Zasięgów i Obrażeń się różnią!".into()
            }
            (RangeDamageError::RangeDamageDiffers, LocaleLang::English) => "Ranges and Damages lengths differ!".into(),
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeDamage {
    pub range: Option<i32>,
    pub damage: String,
}

impl RangeDamage {
    pub fn from(ranges: Vec<String>, damages: Vec<String>) -> Result<Vec<Self>, RangeDamageError> {
        if ranges.len() != damages.len() {
            return Err(RangeDamageError::RangeDamageDiffers);
        }

        Ok(zip(ranges, damages)
            .map(|(range, damage)| Self {
                range: range.parse::<i32>().ok(),
                damage,
            })
            .collect())
    }
}
