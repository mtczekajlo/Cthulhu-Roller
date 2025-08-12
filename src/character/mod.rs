use crate::{
    Error,
    locale::{LocaleEntry, LocaleLang, LocaleTag, locale_entry_by_tag, locale_tag_by_str},
    types::SkillMap,
};
use serde::{Deserialize, Serialize};

pub mod attributes;
pub use attributes::*;
pub mod skill;
pub use skill::*;
pub mod weapon;
pub use weapon::*;
pub mod item;
pub use item::*;

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

pub fn default_skills(attributes: &Attributes) -> Result<SkillMap, Error> {
    let mut skill_map = SkillMap::new();
    for (locale_tag, value) in DEFAULT_SKILLS.iter() {
        let (k, v) = match locale_tag {
            LocaleTag::LanguageOwn => skill_map_wrapper(*locale_tag, attributes.education.value)?,
            LocaleTag::Dodge => skill_map_wrapper(*locale_tag, attributes.dexterity.value / 2)?,
            _ => skill_map_wrapper(*locale_tag, *value)?,
        };
        skill_map.insert(k, v);
    }
    for (locale_tag, value) in DEFAULT_UNIMPROVABLE_SKILLS.iter() {
        let (k, v) = skill_unimprovable_map_wrapper(*locale_tag, *value)?;
        skill_map.insert(k, v);
    }
    for (locale_tag, value) in DEFAULT_CONSTANT_SKILLS.iter() {
        let (k, v) = skill_constant_map_wrapper(*locale_tag, *value)?;
        skill_map.insert(k, v);
    }
    Ok(skill_map)
}

pub fn skill_map_wrapper(tag: LocaleTag, val: i32) -> Result<(String, Skill), Error> {
    Ok((
        locale_entry_by_tag(tag).clone().en,
        Skill::new_default(locale_entry_by_tag(tag).clone(), val),
    ))
}

fn skill_unimprovable_map_wrapper(tag: LocaleTag, val: i32) -> Result<(String, Skill), Error> {
    Ok((
        locale_entry_by_tag(tag).clone().en,
        Skill::new_default_unimprovable(locale_entry_by_tag(tag).clone(), val),
    ))
}

fn skill_constant_map_wrapper(tag: LocaleTag, val: i32) -> Result<(String, Skill), Error> {
    Ok((
        locale_entry_by_tag(tag).clone().en,
        Skill::new_default_constant(locale_entry_by_tag(tag).clone(), val),
    ))
}

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
    pub items: Vec<Item>,
    pub pulp_archetype: Option<LocaleEntry>,
    pub pulp_talents: Vec<LocaleEntry>,
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
    pub fn new(
        name: &str,
        attributes: Attributes,
        luck: i32,
        pulp_archetype: Option<LocaleEntry>,
    ) -> Result<Self, Error> {
        Ok(Character {
            name: name.into(),
            magic: CharacterVariable::new_clamped(attributes.power.value / 5),
            skills: default_skills(&attributes)?,
            move_rate: attributes.calculate_move_rate(),
            hp: CharacterVariable::new_clamped(attributes.calculate_hp(pulp_archetype.is_some())),
            luck: CharacterVariable::new(luck, 99),
            sanity: CharacterVariable::new(attributes.calculate_sanity(), 99),
            build: attributes.calculate_build(),
            attributes,
            dead: false,
            major_wound: false,
            fragile_mind: false,
            insane: false,
            items: vec![],
            weapons: vec![Weapon {
                name: "🙌".into(),
                range_dmgs: RangeDamage::from(vec!["-".into()], vec!["1d3".into()]).unwrap(),
                attacks_query: "1".into(),
                attacks: attacks_from("1")?,
                apply_damage_modifier: true,
                half_damage_modifier: false,
                impaling: false,
                skill: locale_entry_by_tag(LocaleTag::FightingBrawl).en.clone(),
                malfunction: None,
                ammo: None,
                default: true,
            }],
            pulp_archetype,
            pulp_talents: vec![],
        })
    }

    pub fn set_attribute(&mut self, name: &str, value: i32) {
        if let Some(entry) = self.attributes.get_mut(name) {
            entry.value = value;
        }

        if self.attributes.strength.name.equals_ignore_case(name) {
            self.build = self.attributes.calculate_build();
            self.move_rate = self.attributes.calculate_move_rate();
        }
        if self.attributes.constitution.name.equals_ignore_case(name) {
            self.hp.max = self.attributes.calculate_hp(self.pulp_archetype.is_some());
        }
        if self.attributes.size.name.equals_ignore_case(name) {
            self.build = self.attributes.calculate_build();
            self.move_rate = self.attributes.calculate_move_rate();
            self.hp.max = self.attributes.calculate_hp(self.pulp_archetype.is_some());
        }
        if self.attributes.dexterity.name.equals_ignore_case(name) {
            self.move_rate = self.attributes.calculate_move_rate();
        }
        self.attributes.appearance.name.equals_ignore_case(name);
        self.attributes.intelligence.name.equals_ignore_case(name);
        if self.attributes.power.name.equals_ignore_case(name) {
            self.build = self.attributes.calculate_magic();
        }
        if self.attributes.education.name.equals_ignore_case(name) {}
    }

    pub fn get_skill_partial(&self, partial_skill_name: &str) -> Option<Skill> {
        if let Some((_, v)) = self
            .skills
            .iter()
            .find(|(_, v)| v.name.partial_match_ignore_case(partial_skill_name))
        {
            return Some(v.clone());
        }
        None
    }

    pub fn get_skill(&self, skill_name: &str) -> Option<Skill> {
        if let Some((_, v)) = self.skills.iter().find(|(_, v)| v.name.equals_ignore_case(skill_name)) {
            return Some(v.clone());
        }
        None
    }

    pub fn get_mut_skill(&mut self, skill_name: &str) -> Option<&mut Skill> {
        if let Some((_, v)) = self
            .skills
            .iter_mut()
            .find(|(_, v)| v.name.equals_ignore_case(skill_name))
        {
            return Some(v);
        }
        None
    }

    pub fn add_skill(&mut self, name: &str, value: i32) -> Result<(), SkillError> {
        let locale_text = if let Some(tag) = locale_tag_by_str(name) {
            locale_entry_by_tag(tag).clone()
        } else {
            LocaleEntry::new_single_lang(name)
        };
        let new_skill = Skill::new_custom(locale_text.clone(), value);
        match self.skills.get(&locale_text.en) {
            Some(_) => Err(SkillError::AlreadyExists(name.to_string())),
            None => {
                self.skills.insert(locale_text.en, new_skill);
                Ok(())
            }
        }
    }

    pub fn set_skill(&mut self, name: &str, value: i32) -> Result<(), SkillError> {
        let skill = self
            .skills
            .iter_mut()
            .find(|(_, v)| v.name.equals_ignore_case(name))
            .ok_or(SkillError::NoSuchSkill(name.to_string()))?
            .1;
        skill.set(value)?;
        if skill.name.equals_ignore_case("Cthulhu Mythos") {
            self.sanity.max = 99 - skill.value;
        }
        Ok(())
    }

    pub fn remove_skill(&mut self, skill_name: &str) -> Result<(), SkillError> {
        if !self
            .skills
            .iter_mut()
            .any(|(_, skill)| skill.name.equals_ignore_case(skill_name) && !skill.default)
        {
            return Err(SkillError::CantRemove(skill_name.to_string()));
        }

        Ok(())
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
        format!("`{}`\n{}", self.name, self.status())
    }

    pub fn damage_modifier(&self, lang: LocaleLang) -> String {
        let mut out = Self::build_to_damage_modifier(self.build);
        if let LocaleLang::Polski = lang {
            out = out.replace('d', "k")
        }
        out
    }

    fn build_to_damage_modifier(build: i32) -> String {
        let out: String = match build {
            -2 => "-2".to_string(),
            -1 => "-1".to_string(),
            0 => "0".to_string(),
            1 => "+1d4".to_string(),
            v => format!("+{}d6", v - 1),
        };
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(-2,"-2")]
    #[case(-1,"-1")]
    #[case(0, "0")]
    #[case(1, "+1d4")]
    #[case(2, "+1d6")]
    #[case(3, "+2d6")]
    #[case(4, "+3d6")]
    #[case(5, "+4d6")]
    #[case(6, "+5d6")]
    #[case(7, "+6d6")]
    #[case(8, "+7d6")]
    fn test_damage_modifier(#[case] build: i32, #[case] dice: &str) {
        assert_eq!(Character::build_to_damage_modifier(build), dice);
    }
}
