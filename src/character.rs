use crate::{
    locale::{LocaleLang, DEFAULT_CONST_SKILLS, DEFAULT_SKILLS},
    types::{AttributeMap, SkillMap},
    Error,
};

use crate::locale::{locale_text, LocaleTag, LocaleText};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CharacterVariable {
    pub current: i32,
    pub initial: i32,
    pub start: i32,
    pub max: i32,
}

impl CharacterVariable {
    pub fn new(start: i32, max: i32) -> Self {
        Self {
            current: start,
            initial: start,
            start,
            max,
        }
    }

    pub fn new_clamped(start: i32) -> Self {
        Self {
            current: start,
            initial: start,
            start,
            max: start,
        }
    }

    pub fn modify(&mut self, delta: i32) {
        self.current = (self.current + delta).clamp(0, self.max);
    }

    pub fn update_initial(&mut self) {
        self.initial = self.current;
    }

    pub fn reset(&mut self) {
        self.current = self.start;
        self.initial = self.start;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Attribute {
    pub name: LocaleText,
    pub value: i32,
}

impl Attribute {
    pub fn new(text: LocaleText, value: i32) -> Self {
        Self { name: text, value }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Skill {
    pub name: LocaleText,
    pub value: i32,
    pub to_improve: bool,
    pub improvable: bool,
    pub default: bool,
}

impl Skill {
    pub fn new_default(text: LocaleText, value: i32) -> Self {
        Self {
            name: text,
            value,
            improvable: true,
            default: true,
            ..Default::default()
        }
    }

    pub fn new_default_const(text: LocaleText, value: i32) -> Self {
        Self {
            name: text,
            value,
            default: true,
            ..Default::default()
        }
    }

    pub fn new_custom(text: LocaleText, value: i32) -> Self {
        Self {
            name: text,
            value,
            improvable: true,
            ..Default::default()
        }
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
        map.insert(locale_text(&LocaleTag::Strength).unwrap().clone().en, value.strength);
        map.insert(
            locale_text(&LocaleTag::Constitution).unwrap().clone().en,
            value.constitution,
        );
        map.insert(locale_text(&LocaleTag::Size).unwrap().clone().en, value.size);
        map.insert(locale_text(&LocaleTag::Dexterity).unwrap().clone().en, value.dexterity);
        map.insert(
            locale_text(&LocaleTag::Appearance).unwrap().clone().en,
            value.appearance,
        );
        map.insert(
            locale_text(&LocaleTag::Intelligence).unwrap().clone().en,
            value.intelligence,
        );
        map.insert(locale_text(&LocaleTag::Power).unwrap().clone().en, value.power);
        map.insert(locale_text(&LocaleTag::Education).unwrap().clone().en, value.education);
        map
    }
}

#[allow(clippy::too_many_arguments)]
impl Attributes {
    pub fn new(str: i32, con: i32, siz: i32, dex: i32, app: i32, int: i32, pow: i32, edu: i32) -> Result<Self, Error> {
        Ok(Self {
            strength: Attribute::new(locale_text(&LocaleTag::Strength)?.clone(), str),
            constitution: Attribute::new(locale_text(&LocaleTag::Constitution)?.clone(), con),
            size: Attribute::new(locale_text(&LocaleTag::Size)?.clone(), siz),
            dexterity: Attribute::new(locale_text(&LocaleTag::Dexterity)?.clone(), dex),
            appearance: Attribute::new(locale_text(&LocaleTag::Appearance)?.clone(), app),
            intelligence: Attribute::new(locale_text(&LocaleTag::Intelligence)?.clone(), int),
            power: Attribute::new(locale_text(&LocaleTag::Power)?.clone(), pow),
            education: Attribute::new(locale_text(&LocaleTag::Education)?.clone(), edu),
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
        .find(|a| a.name.equals(attribute_name))
        .cloned()
    }

    pub fn get_mut(&mut self, attribute_name: &str) -> Option<&mut Attribute> {
        if self.strength.name.equals(attribute_name) {
            Some(&mut self.strength)
        } else if self.constitution.name.equals(attribute_name) {
            Some(&mut self.constitution)
        } else if self.size.name.equals(attribute_name) {
            Some(&mut self.size)
        } else if self.dexterity.name.equals(attribute_name) {
            Some(&mut self.dexterity)
        } else if self.appearance.name.equals(attribute_name) {
            Some(&mut self.appearance)
        } else if self.intelligence.name.equals(attribute_name) {
            Some(&mut self.intelligence)
        } else if self.power.name.equals(attribute_name) {
            Some(&mut self.power)
        } else if self.education.name.equals(attribute_name) {
            Some(&mut self.education)
        } else {
            None
        }
    }

    pub fn calculate_build(&self) -> i32 {
        match self.strength.value + self.size.value {
            0..65 => -2,
            65..85 => -1,
            85..125 => 0,
            125..165 => 1,
            165..999 => 2,
            _ => todo!(),
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

    pub fn calculate_hp(&self) -> i32 {
        (self.constitution.value + self.size.value) / 10
    }

    pub fn calculate_sanity(&self) -> i32 {
        self.power.value
    }

    pub fn calculate_magic(&self) -> i32 {
        self.power.value / 5
    }
}

pub fn default_skills(attributes: &Attributes) -> Result<SkillMap, Error> {
    let mut skill_map = SkillMap::new();
    for (locale_tag, value) in DEFAULT_SKILLS.iter() {
        let (k, v) = match locale_tag {
            LocaleTag::LanguageOwn => skill_map_wrapper(locale_tag, attributes.education.value)?,
            LocaleTag::Dodge => skill_map_wrapper(locale_tag, attributes.dexterity.value / 2)?,
            _ => skill_map_wrapper(locale_tag, *value)?,
        };
        skill_map.insert(k, v);
    }
    for (locale_tag, value) in DEFAULT_CONST_SKILLS.iter() {
        let (k, v) = skill_const_map_wrapper(locale_tag, *value)?;
        skill_map.insert(k, v);
    }
    Ok(skill_map)
}

fn skill_map_wrapper(tag: &LocaleTag, val: i32) -> Result<(String, Skill), Error> {
    Ok((
        locale_text(tag)?.clone().en,
        Skill::new_default(locale_text(tag)?.clone(), val),
    ))
}

fn skill_const_map_wrapper(tag: &LocaleTag, val: i32) -> Result<(String, Skill), Error> {
    Ok((
        locale_text(tag)?.clone().en,
        Skill::new_default_const(locale_text(tag)?.clone(), val),
    ))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Weapon {
    pub name: LocaleText,
    pub dmg: String,
    pub apply_damage_modifier: bool,
    pub half_damage_modifier: bool,
    pub impaling: bool,
    pub skill: String,
    pub range: Option<i32>,
    pub malfunction: Option<i32>,
    pub default: bool,
}

impl Weapon {
    pub fn new(
        name: &str,
        dmg: &str,
        apply_damage_modifier: bool,
        impaling: bool,
        skill: &str,
        range: Option<i32>,
        malfunction: Option<i32>,
    ) -> Self {
        let dmg = dmg.replace("k", "d").replace(" ", "");
        Self {
            name: LocaleText::new_single_lang(name),
            dmg,
            apply_damage_modifier,
            half_damage_modifier: false,
            impaling,
            skill: skill.to_string(),
            malfunction,
            range,
            default: false,
        }
    }

    pub fn damage(&self, lang: &LocaleLang) -> String {
        let mut dmg = self.dmg.clone();
        if lang == &LocaleLang::Polski {
            dmg = dmg.replace("d", "k");
        }
        dmg
    }

    pub fn apply_half_dm(&mut self) {
        self.half_damage_modifier = true;
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Character {
    pub name: String,
    pub attributes: Attributes,
    pub build: i32,
    pub move_rate: i32,
    pub hp: CharacterVariable,
    pub major_wound: bool,
    pub dead: bool,
    pub sanity: CharacterVariable,
    pub fragile_mind: bool,
    pub insane: bool,
    pub luck: CharacterVariable,
    pub magic: CharacterVariable,
    pub skills: SkillMap,
    pub weapons: Vec<Weapon>,
    pub items: Vec<String>,
}

impl PartialOrd for Character {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Character {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Character {}

impl Character {
    pub fn new(name: &str, attributes: Attributes, luck: i32) -> Result<Self, Error> {
        Ok(Character {
            name: name.into(),
            magic: CharacterVariable::new_clamped(attributes.power.value / 5),
            skills: default_skills(&attributes)?,
            move_rate: attributes.calculate_move_rate(),
            hp: CharacterVariable::new_clamped(attributes.calculate_hp()),
            luck: CharacterVariable::new_clamped(luck),
            sanity: CharacterVariable::new(attributes.calculate_sanity(), 99),
            build: attributes.calculate_build(),
            attributes,
            dead: false,
            major_wound: false,
            fragile_mind: false,
            insane: false,
            items: vec![],
            weapons: vec![Weapon {
                name: locale_text(&LocaleTag::Unarmed)?.clone(),
                dmg: "d3".to_string(),
                apply_damage_modifier: true,
                half_damage_modifier: false,
                impaling: false,
                skill: locale_text(&LocaleTag::FightingBrawl)?.en.clone(),
                range: None,
                malfunction: None,
                default: true,
            }],
        })
    }

    pub fn set_attribute(&mut self, name: &str, value: i32) {
        if let Some(entry) = self.attributes.get_mut(name) {
            entry.value = value;
        }

        if self.attributes.strength.name.equals(name) {
            self.build = self.attributes.calculate_build();
            self.move_rate = self.attributes.calculate_move_rate();
        }
        if self.attributes.constitution.name.equals(name) {
            self.hp.max = self.attributes.calculate_hp();
        }
        if self.attributes.size.name.equals(name) {
            self.build = self.attributes.calculate_build();
            self.move_rate = self.attributes.calculate_move_rate();
            self.hp.max = self.attributes.calculate_hp();
        }
        if self.attributes.dexterity.name.equals(name) {
            self.move_rate = self.attributes.calculate_move_rate();
        }
        self.attributes.appearance.name.equals(name);
        self.attributes.intelligence.name.equals(name);
        if self.attributes.power.name.equals(name) {
            self.build = self.attributes.calculate_magic();
        }
        if self.attributes.education.name.equals(name) {}
    }

    pub fn get_skill_partial(&self, partial_skill_name: &str) -> Option<Skill> {
        if let Some((_, v)) = self
            .skills
            .iter()
            .find(|(_, v)| v.name.partial_match(partial_skill_name))
        {
            return Some(v.clone());
        }
        None
    }

    pub fn get_skill(&self, skill_name: &str) -> Option<Skill> {
        if let Some((_, v)) = self.skills.iter().find(|(_, v)| v.name.equals(skill_name)) {
            return Some(v.clone());
        }
        None
    }

    pub fn get_mut_skill(&mut self, skill_name: &str) -> Option<&mut Skill> {
        if let Some((_, v)) = self.skills.iter_mut().find(|(_, v)| v.name.equals(skill_name)) {
            return Some(v);
        }
        None
    }

    pub fn add_skill(&mut self, name: &str, value: i32) -> Result<(), Error> {
        let new_skill = Skill::new_custom(LocaleText::new_single_lang(name), value);
        match self.skills.get(name) {
            Some(_) => Err("Skill already exists!".into()),
            None => {
                self.skills
                    .insert(name.to_string(), new_skill)
                    .ok_or("Couldn't add skill")?;
                Ok(())
            }
        }
    }

    pub fn set_skill(&mut self, name: &str, value: i32) -> Result<(), Error> {
        let skill = self
            .skills
            .iter_mut()
            .find(|(_, v)| v.name.equals(name))
            .ok_or(format!("No such skill {name}"))?
            .1;
        skill.value = value;
        if skill.name.equals("Cthulhu Mythos") {
            self.sanity.max = 99 - skill.value;
        }
        Ok(())
    }

    pub fn delete_skill(&mut self, skill_name: &str) {
        self.skills.retain(|_, v| v.default || !(v.name.equals(skill_name)));
    }

    pub fn reset(&mut self) {
        self.dead = false;
        self.major_wound = false;
        self.insane = false;
        self.fragile_mind = false;
        self.hp.reset();
        self.sanity.reset();
        self.luck.reset();
        self.magic.reset();
    }

    pub fn name(&self) -> String {
        format!("`{}`", self.name)
    }

    pub fn name_raw(&self) -> String {
        self.name.to_string()
    }

    pub fn status_hp(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.dead {
            comment = Some(" 💀");
        } else if self.major_wound {
            comment = Some(" 🤕");
        }

        format!(
            "❤️ **{}**/{}{}",
            self.hp.current,
            self.hp.max,
            comment.unwrap_or_default(),
        )
    }

    pub fn status_hp_raw(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.dead {
            comment = Some(" 💀");
        } else if self.major_wound {
            comment = Some(" 🤕");
        }

        format!("{}/{}{}", self.hp.current, self.hp.max, comment.unwrap_or_default(),)
    }

    pub fn status_sanity(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.insane {
            comment = Some(" 🤯")
        } else if self.fragile_mind {
            comment = Some(" 😨")
        }

        format!("🧠 **{}**{}", self.sanity.current, comment.unwrap_or_default(),)
    }

    pub fn status_sanity_raw(&self) -> String {
        let mut comment: Option<&str> = None;
        if self.insane {
            comment = Some(" 🤯")
        } else if self.fragile_mind {
            comment = Some(" 😨")
        }

        format!("{}{}", self.sanity.current, comment.unwrap_or_default(),)
    }

    pub fn status_luck(&self) -> String {
        format!("🍀 **{}**", self.luck.current)
    }

    pub fn status_luck_raw(&self) -> String {
        format!("{}", self.luck.current)
    }

    pub fn status_magic(&self) -> String {
        format!("🪄 **{}**/{}", self.magic.current, self.magic.max)
    }

    pub fn status_magic_raw(&self) -> String {
        format!("{}/{}", self.magic.current, self.magic.max)
    }

    pub fn status(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.status_hp(),
            self.status_sanity(),
            self.status_luck(),
            self.status_magic()
        )
    }

    pub fn status_one_line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.status_hp(),
            self.status_sanity(),
            self.status_luck(),
            self.status_magic()
        )
    }

    pub fn status_named(&self) -> String {
        format!("{}\n{}", self.name(), self.status())
    }

    pub fn damage_modifier(&self, lang: &LocaleLang) -> String {
        let mut out: String = match self.build {
            -2 => "-2",
            -1 => "-1",
            0 => "0",
            1 => "+d4",
            2 => "+d6",
            _ => todo!(),
        }
        .into();
        if let LocaleLang::Polski = lang {
            out = out.replace('d', "k")
        }
        out
    }
}
