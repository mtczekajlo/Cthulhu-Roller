use crate::{
    Error,
    locale::{LocaleEntry, LocaleTag, locale_entry_by_tag},
    types::AttributeMap,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Attribute {
    pub name: LocaleEntry,
    pub value: i32,
}

impl Attribute {
    pub fn new(text: LocaleEntry, value: i32) -> Self {
        Self { name: text, value }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Attributes {
    pub strength: Attribute,
    pub constitution: Attribute,
    pub size: Attribute,
    pub dexterity: Attribute,
    pub appearance: Attribute,
    pub intelligence: Attribute,
    pub power: Attribute,
    pub education: Attribute,
}

impl From<Attributes> for AttributeMap {
    fn from(value: Attributes) -> Self {
        let mut map = AttributeMap::new();
        map.insert(locale_entry_by_tag(LocaleTag::Strength).clone().en, value.strength);
        map.insert(
            locale_entry_by_tag(LocaleTag::Constitution).clone().en,
            value.constitution,
        );
        map.insert(locale_entry_by_tag(LocaleTag::Size).clone().en, value.size);
        map.insert(locale_entry_by_tag(LocaleTag::Dexterity).clone().en, value.dexterity);
        map.insert(locale_entry_by_tag(LocaleTag::Appearance).clone().en, value.appearance);
        map.insert(
            locale_entry_by_tag(LocaleTag::Intelligence).clone().en,
            value.intelligence,
        );
        map.insert(locale_entry_by_tag(LocaleTag::Power).clone().en, value.power);
        map.insert(locale_entry_by_tag(LocaleTag::Education).clone().en, value.education);
        map
    }
}

#[allow(clippy::too_many_arguments)]
impl Attributes {
    pub fn new(str: i32, con: i32, siz: i32, dex: i32, app: i32, int: i32, pow: i32, edu: i32) -> Result<Self, Error> {
        Ok(Self {
            strength: Attribute::new(locale_entry_by_tag(LocaleTag::Strength).clone(), str),
            constitution: Attribute::new(locale_entry_by_tag(LocaleTag::Constitution).clone(), con),
            size: Attribute::new(locale_entry_by_tag(LocaleTag::Size).clone(), siz),
            dexterity: Attribute::new(locale_entry_by_tag(LocaleTag::Dexterity).clone(), dex),
            appearance: Attribute::new(locale_entry_by_tag(LocaleTag::Appearance).clone(), app),
            intelligence: Attribute::new(locale_entry_by_tag(LocaleTag::Intelligence).clone(), int),
            power: Attribute::new(locale_entry_by_tag(LocaleTag::Power).clone(), pow),
            education: Attribute::new(locale_entry_by_tag(LocaleTag::Education).clone(), edu),
        })
    }

    pub fn get(&self, attribute_name: &str) -> Option<&Attribute> {
        [
            &self.strength,
            &self.constitution,
            &self.size,
            &self.dexterity,
            &self.appearance,
            &self.intelligence,
            &self.power,
            &self.education,
        ]
        .iter()
        .find(|a| a.name.equals_ignore_case(attribute_name))
        .cloned()
    }

    pub fn get_mut(&mut self, attribute_name: &str) -> Option<&mut Attribute> {
        if self.strength.name.equals_ignore_case(attribute_name) {
            Some(&mut self.strength)
        } else if self.constitution.name.equals_ignore_case(attribute_name) {
            Some(&mut self.constitution)
        } else if self.size.name.equals_ignore_case(attribute_name) {
            Some(&mut self.size)
        } else if self.dexterity.name.equals_ignore_case(attribute_name) {
            Some(&mut self.dexterity)
        } else if self.appearance.name.equals_ignore_case(attribute_name) {
            Some(&mut self.appearance)
        } else if self.intelligence.name.equals_ignore_case(attribute_name) {
            Some(&mut self.intelligence)
        } else if self.power.name.equals_ignore_case(attribute_name) {
            Some(&mut self.power)
        } else if self.education.name.equals_ignore_case(attribute_name) {
            Some(&mut self.education)
        } else {
            None
        }
    }

    pub fn calculate_build(&self) -> i32 {
        let str_siz = self.strength.value + self.size.value;
        match str_siz {
            0..=64 => -2,
            65..=84 => -1,
            85..=124 => 0,
            125..=164 => 1,
            165..=204 => 2,
            v => 2 + (v - 204 + 79) / 80,
        }
    }

    pub fn calculate_move_rate(&self) -> i32 {
        if self.strength.value.max(self.dexterity.value) < self.size.value {
            7
        } else if self.strength.value.min(self.dexterity.value) > self.size.value {
            9
        } else {
            8
        }
    }

    pub fn calculate_hp(&self, pulp: bool) -> i32 {
        let divider = if pulp { 5 } else { 10 };
        (self.constitution.value + self.size.value) / divider
    }

    pub fn calculate_sanity(&self) -> i32 {
        self.power.value
    }

    pub fn calculate_magic(&self) -> i32 {
        self.power.value / 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(2, -2)]
    #[case(64, -2)]
    #[case(65, -1)]
    #[case(84, -1)]
    #[case(85, 0)]
    #[case(124, 0)]
    #[case(125, 1)]
    #[case(164, 1)]
    #[case(165, 2)]
    #[case(204, 2)]
    #[case(205, 3)]
    #[case(284, 3)]
    #[case(285, 4)]
    #[case(364, 4)]
    #[case(365, 5)]
    #[case(444, 5)]
    #[case(445, 6)]
    #[case(524, 6)]
    #[case(525, 7)]
    #[case(604, 7)]
    #[case(605, 8)]
    #[case(684, 8)]
    #[case(685, 9)]
    fn test_build(#[case] str: i32, #[case] build: i32) {
        let a = Attributes::new(str, 0, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(a.calculate_build(), build);
    }
}
