use std::collections::HashMap;

use crate::{
    locale::{LocaleTag, locale_tag_by_str},
    roller::{
        dice_rng::DiceRng,
        roll::{RollResult, roll_dice_no_query},
    },
};

pub struct AttributeRollResult {
    pub roll_map: HashMap<LocaleTag, RollResult>,
    is_pulp: bool,
}

impl AttributeRollResult {
    pub fn new<D: DiceRng>(rng: &mut D, pulp_core_attribute: Option<&str>) -> Self {
        let mut rolls: HashMap<LocaleTag, RollResult> = [
            (LocaleTag::Strength, roll_dice_no_query(rng, 3, 6, 1.0, 0)),
            (LocaleTag::Constitution, roll_dice_no_query(rng, 3, 6, 1.0, 0)),
            (LocaleTag::Size, roll_dice_no_query(rng, 2, 6, 1.0, 6)),
            (LocaleTag::Dexterity, roll_dice_no_query(rng, 3, 6, 1.0, 0)),
            (LocaleTag::Appearance, roll_dice_no_query(rng, 3, 6, 1.0, 0)),
            (LocaleTag::Intelligence, roll_dice_no_query(rng, 2, 6, 1.0, 6)),
            (LocaleTag::Power, roll_dice_no_query(rng, 3, 6, 1.0, 0)),
            (LocaleTag::Education, roll_dice_no_query(rng, 2, 6, 1.0, 6)),
        ]
        .iter()
        .cloned()
        .collect();

        let mut is_pulp = false;
        if let Some(pulp_core_attribute) = &pulp_core_attribute {
            is_pulp = true;
            let tag_to_find = locale_tag_by_str(pulp_core_attribute)
                .unwrap_or_else(|| panic!("No tag for string: {pulp_core_attribute:?}"));
            let (_, core_attribute_result) = rolls
                .iter_mut()
                .find(|el| *el.0 == tag_to_find)
                .unwrap_or_else(|| panic!(""));
            *core_attribute_result = roll_dice_no_query(rng, 1, 6, 1.0, 13);
        }

        Self {
            roll_map: rolls,
            is_pulp,
        }
    }

    pub fn quick_rules_pts(&self) -> i32 {
        if self.is_pulp { 500 } else { 460 }
    }

    pub fn points_sum(&self) -> i32 {
        self.roll_map.values().map(|dr| dr.result() * 5).sum::<i32>()
    }

    pub fn is_sum_eq_quick_rules(&self) -> bool {
        self.points_sum() == self.quick_rules_pts()
    }

    pub fn is_sum_lt_quick_rules(&self) -> bool {
        self.points_sum() < self.quick_rules_pts()
    }

    pub fn lowest_attribute_value(&self) -> i32 {
        self.roll_map.iter().map(|el| el.1.result() * 5).min().unwrap()
    }
}
