#[cfg(feature = "character-sheet")]
use crate::character::{Character, Item, Skill};
use crate::roller::attribute_roll::AttributeRollResult;
use crate::roller::croll::CrollResult;
use crate::roller::improve_roll::ImproveResult;
use crate::roller::roll::RollResult;
#[cfg(feature = "character-sheet")]
use crate::types::SkillMap;
#[cfg(feature = "character-sheet")]
use crate::utils::to_uppercase_first_letter;
use crate::{
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    roller::{battle::Battle, success_level::SuccessLevel},
};
#[cfg(feature = "character-sheet")]
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter, Embed};
#[cfg(feature = "character-sheet")]
use tabled::builder::Builder;
use tabled::settings::Style;
#[cfg(feature = "character-sheet")]
use tabled::settings::{Alignment, Modify, object::Column};
pub mod help;

#[derive(Default, Clone)]
pub struct MessageContent {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub colour: Option<u32>,
}

impl MessageContent {
    pub fn to_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed = embed.title(&self.title);
        embed = embed.description(&self.description);
        embed = embed.footer(CreateEmbedFooter::new(&self.footer));
        embed = embed.colour(match self.colour {
            Some(colour) => Colour::from(colour),
            None => Colour::from(SuccessLevel::Success.hex()),
        });
        embed
    }

    #[cfg(feature = "character-sheet")]
    pub fn to_content(&self) -> String {
        let mut out = String::new();
        if !self.title.is_empty() {
            out.push_str(&format!("**{}**\n\n", self.title));
        }
        out.push_str(&self.description);
        out
    }

    pub fn from_dice_result(lang: LocaleLang, roll_result: RollResult, hide_details: bool) -> Self {
        let mut mc = Self {
            title: format!("**{}**", roll_result.result()),
            ..Default::default()
        };

        if !hide_details {
            mc.description = format!(
                "{} {}",
                locale_text_by_tag_lang(lang, LocaleTag::Rolls),
                roll_result.roll_msg
            );

            if roll_result.result_real() < 0 {
                mc.description = format!("{}\n*({})*", mc.description, roll_result.result_real())
            };

            mc.footer = roll_result.query;
            if mc.footer.starts_with('+') {
                mc.footer = mc.footer.replacen('+', "", 1);
            }
            if lang == LocaleLang::Polski {
                mc.footer = mc.footer.replace("d", "k");
            }
        }

        mc
    }

    pub fn from_croll_result(
        lang: LocaleLang,
        croll_result: &CrollResult,
        hide_details: bool,
        hide_luck: bool,
    ) -> Self {
        let mut mc = Self {
            title: format!(
                "**{}**",
                locale_text_by_tag_lang(lang, croll_result.success_level.to_locale_tag()),
            ),
            colour: Some(croll_result.success_level.hex()),
            ..Default::default()
        };

        if !hide_details {
            let mut description = format!(
                "**{}** / {}\n{} {} `[{}]`",
                croll_result.result(),
                croll_result.threshold,
                locale_text_by_tag_lang(lang, LocaleTag::Rolls),
                croll_result
                    .ten_rolls
                    .iter()
                    .fold(String::new(), |s, el| format!("{s} `[{el}0]`")),
                croll_result.one_roll
            );

            if let Some(modifier_dice) = &croll_result.modifier_dice {
                description = format!(
                    "{}\n{} {}: {}",
                    description,
                    locale_text_by_tag_lang(lang, modifier_dice.dice_type.to_locale_tag()),
                    locale_text_by_tag_lang(lang, LocaleTag::Dice),
                    modifier_dice.count
                );
            }

            if !hide_luck {
                let mut previous_delta = 0;
                for success_level in croll_result.success_level {
                    let luck_delta = success_level.delta(croll_result.result(), croll_result.threshold);
                    if previous_delta == luck_delta {
                        break;
                    }
                    previous_delta = luck_delta;
                    description = format!(
                        "{}\n🍀 {} ➡️ {}",
                        description,
                        luck_delta,
                        locale_text_by_tag_lang(lang, success_level.to_locale_tag()),
                    );
                }
            }

            mc.description = description;
        }

        mc
    }

    #[cfg(feature = "character-sheet")]
    pub fn with_skill_name(mut self, skill_name: &str) -> Self {
        self.title.push_str(format!("\n{skill_name}").as_str());
        self
    }

    #[cfg(feature = "character-sheet")]
    pub fn with_character_name(mut self, name: &str) -> Self {
        self.title.push_str(format!(" (`{name}`)").as_str());
        self
    }

    pub fn from_battle(lang: LocaleLang, battle: &Battle, hide_details: bool) -> Self {
        let mut current_character_name = None;
        let mut table = tabled::builder::Builder::new();

        let mut header = vec![];
        header.extend([
            "".into(),
            "#".into(),
            locale_text_by_tag_lang(lang, LocaleTag::Name),
            locale_text_by_tag_lang(lang, LocaleTag::Result),
        ]);
        if !hide_details {
            header.extend([
                locale_text_by_tag_lang(lang, LocaleTag::Dex),
                locale_text_by_tag_lang(lang, LocaleTag::Rolls),
            ]);
        }
        table.push_record(header);

        for (i, character) in battle.characters.iter().enumerate() {
            let result = match character.croll_result.success_level {
                SuccessLevel::CriticalSuccess => {
                    format!(
                        "\n^ {}",
                        locale_text_by_tag_lang(lang, LocaleTag::BonusDieToFirstAction)
                    )
                }
                SuccessLevel::CriticalFailure => {
                    format!("\n^ {}", locale_text_by_tag_lang(lang, LocaleTag::LosesFirstRound))
                }
                _ => "".into(),
            };
            let result = format!(
                "{}{}",
                locale_text_by_tag_lang(lang, character.croll_result.success_level.to_locale_tag()),
                result
            );

            let mut row: Vec<String> = vec![];

            if battle.current_position == i {
                row.extend(["➡️".into()]);
                current_character_name = Some(character.name.clone());
            } else {
                row.extend(["".into()]);
            }

            row.extend([(i + 1).to_string(), character.name.clone(), result]);
            if !hide_details {
                row.extend([
                    character.croll_result.threshold.to_string(),
                    character.croll_result.result().to_string(),
                ]);
            }
            table.push_record(row);
        }

        let table = table.build().with(Style::empty()).to_string();

        let mut out = String::new();
        if let Some(current_character_name) = current_character_name {
            out.push_str(format!("➡️ **{current_character_name}**\n").as_str());
        }
        out.push_str("```text\n");
        out.push_str(table.as_str());
        out.push_str("```");

        Self {
            title: locale_text_by_tag_lang(lang, LocaleTag::Fight),
            description: out,
            ..Default::default()
        }
    }

    pub fn from_improve(lang: LocaleLang, improve_result: &ImproveResult) -> Self {
        Self {
            title: format!(
                "**{}**",
                locale_text_by_tag_lang(lang, improve_result.success_level.to_locale_tag()),
            ),
            colour: Some(improve_result.success_level.hex()),
            description: format!("**{}** / {}", improve_result.result, improve_result.threshold),
            ..Default::default()
        }
    }

    pub fn from_levels(threshold: i32) -> Self {
        Self {
            title: format!("**{} / {} / {}**", threshold, threshold / 2, threshold / 5),
            colour: None,
            description: "".into(),
            ..Default::default()
        }
    }

    pub fn from_attributes_result(lang: LocaleLang, attribute_roll_result: AttributeRollResult) -> Self {
        let mut out = String::new();
        out.push_str("```text\n");

        let mut table = tabled::builder::Builder::new();

        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Characteristic),
            locale_text_by_tag_lang(lang, LocaleTag::Value),
            locale_text_by_tag_lang(lang, LocaleTag::Rolls),
        ]);

        for (tag, dr) in &attribute_roll_result
            .roll_map
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.1.result(), &b.1.result()))
            .rev()
            .collect::<Vec<_>>()
        {
            table.push_record([
                locale_text_by_tag_lang(lang, **tag).as_str(),
                format!("{} ({}/{})", &dr.result() * 5, &dr.result() * 5 / 2, &dr.result()).as_str(),
                format!(
                    "[{}{}] ({})",
                    dr.rolls.iter().join(", "),
                    if (dr.modifier) == 0 {
                        "".to_string()
                    } else {
                        format!(", {}", dr.modifier)
                    },
                    &dr.result()
                )
                .as_str(),
            ]);
        }

        let table = table.build().with(Style::empty()).to_string();
        out.push_str(table.as_str());
        out.push_str("```");

        MessageContent {
            description: out,
            footer: format!(
                "{}/{}",
                attribute_roll_result.points_sum(),
                attribute_roll_result.quick_rules_pts()
            ),
            colour: Some(if !attribute_roll_result.is_sum_lt_quick_rules() {
                SuccessLevel::Success.hex()
            } else {
                SuccessLevel::Failure.hex()
            }),
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_skills(lang: LocaleLang, character_skills: &SkillMap) -> Vec<Self> {
        let mut outs: Vec<_> = vec![];

        let mut skill_list: Vec<(String, Skill)> = character_skills
            .clone()
            .into_iter()
            .map(|el| (el.1.name.get(lang), el.1))
            .collect();
        skill_list.sort();

        let skill_per_message = 48;
        let columns = 2;

        let mut skill_chunks: Vec<Vec<(String, Skill)>> =
            skill_list.chunks(skill_per_message).map(|el| el.to_vec()).collect();

        for skill_chunk in skill_chunks.iter_mut() {
            let records_per_column = skill_chunk.len() / columns + skill_chunk.len() % columns;
            let tail = skill_chunk.split_off(records_per_column);
            let mut out = String::new();

            out.push_str("```text\n");

            let mut table = tabled::builder::Builder::new();

            for pair in skill_chunk.iter().zip_longest(tail.iter()) {
                match pair {
                    Both(left, right) => table.push_record([
                        if left.1.to_improve { "☑" } else { "☐" },
                        left.0.as_str(),
                        left.1.value.to_string().as_str(),
                        if right.1.to_improve { "☑" } else { "☐" },
                        right.0.as_str(),
                        right.1.value.to_string().as_str(),
                    ]),
                    Left(left) => table.push_record([
                        if left.1.to_improve { "☑" } else { "☐" },
                        left.0.as_str(),
                        left.1.value.to_string().as_str(),
                        "",
                        "",
                        "",
                    ]),
                    Right(right) => table.push_record([
                        "",
                        "",
                        "",
                        if right.1.to_improve { "☑" } else { "☐" },
                        right.0.as_str(),
                        right.1.value.to_string().as_str(),
                    ]),
                }
            }

            let mut table = table.build();

            table.with(Style::empty());

            let alignments = [
                (1, tabled::settings::Alignment::left()),
                (2, tabled::settings::Alignment::right()),
                (4, tabled::settings::Alignment::left()),
                (5, tabled::settings::Alignment::right()),
            ];

            for (index, align) in alignments {
                table.with(tabled::settings::Modify::new(tabled::settings::object::Column::from(index)).with(align));
            }

            out.push_str(table.to_string().as_str());
            out.push_str("```\n");
            outs.push(out);
        }

        outs.iter()
            .map(|o| MessageContent {
                description: o.clone(),
                ..Default::default()
            })
            .collect()
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_items(lang: LocaleLang, character_items: &[Item]) -> Self {
        let mut out = String::new();

        let mut items = character_items.to_owned();
        items.sort();

        out.push_str(format!("{}\n```text\n", locale_text_by_tag_lang(lang, LocaleTag::Items)).as_str());

        if items.is_empty() {
            out.push_str(format!("({})\n", locale_text_by_tag_lang(lang, LocaleTag::NoItems)).as_str());
        } else {
            let mut table = tabled::builder::Builder::new();

            let columns = 2;
            let records_per_column = items.len() / columns + items.len() % columns;
            let tail = items.split_off(records_per_column);

            for pair in items.iter().zip_longest(tail.iter()) {
                match pair {
                    Both(left, right) => table.push_record([
                        &("• ".to_string() + left.to_string(lang).as_str()),
                        &("• ".to_string() + right.to_string(lang).as_str()),
                    ]),

                    Left(left) => table.push_record([&("• ".to_string() + left.to_string(lang).as_str()), ""]),

                    Right(right) => table.push_record(["", &("• ".to_string() + right.to_string(lang).as_str())]),
                }
            }

            let mut table = table.build();
            table.with(Style::empty());
            out.push_str(table.to_string().as_str());
        }
        out.push_str("```\n");

        Self {
            description: out,
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_to_items(lang: LocaleLang, character: &Character) -> Self {
        let mut out = String::new();

        let mut items: Vec<_> = character.items.clone();
        items.sort();

        out.push_str(format!("{}\n```text\n", locale_text_by_tag_lang(lang, LocaleTag::Items)).as_str());

        if items.is_empty() {
            out.push_str(format!("({})\n", locale_text_by_tag_lang(lang, LocaleTag::NoItems)).as_str());
        } else {
            let mut table = Builder::new();

            let columns = 2;
            let records_per_column = items.len() / columns + items.len() % columns;
            let tail = items.split_off(records_per_column);

            for pair in items.iter().zip_longest(tail.iter()) {
                match pair {
                    Both(left, right) => table.push_record([
                        &("• ".to_string() + left.to_string(lang).as_str()),
                        &("• ".to_string() + right.to_string(lang).as_str()),
                    ]),

                    Left(left) => table.push_record([&("• ".to_string() + left.to_string(lang).as_str()), ""]),

                    Right(right) => table.push_record(["", &("• ".to_string() + right.to_string(lang).as_str())]),
                }
            }

            let mut table = table.build();
            table.with(Style::empty());
            out.push_str(table.to_string().as_str());
        }
        out.push_str("```");

        Self {
            description: out,
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_to_weapons(lang: LocaleLang, character: &Character) -> Self {
        let mut out = String::new();

        let mut weapon_list: Vec<_> = character.weapons.clone();
        weapon_list.sort();

        out.push_str(format!("{}\n", locale_text_by_tag_lang(lang, LocaleTag::Weapons)).as_str());
        out.push_str("```text\n");

        if weapon_list.is_empty() {
            out.push_str(format!("({})\n", locale_text_by_tag_lang(lang, LocaleTag::NoItems)).as_str());
        } else {
            let mut table = tabled::builder::Builder::new();

            table.push_record([
                to_uppercase_first_letter(&locale_text_by_tag_lang(lang, LocaleTag::Weapon)),
                locale_text_by_tag_lang(lang, LocaleTag::Attacks),
                locale_text_by_tag_lang(lang, LocaleTag::Damage),
                locale_text_by_tag_lang(lang, LocaleTag::Range),
                locale_text_by_tag_lang(lang, LocaleTag::Malfunction),
                locale_text_by_tag_lang(lang, LocaleTag::Ammo),
                locale_text_by_tag_lang(lang, LocaleTag::Skill),
            ]);

            for weapon in weapon_list {
                let mut weapon_dmg = weapon.range_dmgs.iter().map(|rd| &rd.damage).join("/");
                match lang {
                    LocaleLang::English => weapon_dmg = weapon_dmg.replace('k', "d"),
                    LocaleLang::Polski => weapon_dmg = weapon_dmg.replace('d', "k"),
                }
                let dmg = format!(
                    "{}{}",
                    weapon_dmg,
                    if weapon.apply_damage_modifier && character.build != 0 {
                        format!(
                            "+{}{} ({}{})",
                            if weapon.half_damage_modifier { "½" } else { "" },
                            locale_text_by_tag_lang(lang, LocaleTag::Db),
                            character.damage_modifier(lang),
                            if weapon.half_damage_modifier { "x½" } else { "" },
                        )
                    } else {
                        "".to_string()
                    }
                );

                let range = weapon
                    .range_dmgs
                    .iter()
                    .map(|rd| rd.range.map_or("-".to_string(), |r| format!("{} m", r)))
                    .join("/");

                let weapon_name = weapon.name;
                let weapon_skill = character.get_skill(&weapon.skill).unwrap();

                table.push_record([
                    weapon_name,
                    weapon.attacks_query,
                    dmg,
                    range,
                    weapon.malfunction.map_or("-".to_string(), |n| n.to_string()),
                    weapon.ammo.clone().map_or("-".to_string(), |a| {
                        format!("{}/{} [{}]", a.clip_rounds, a.clip_size, a.loose_rounds)
                    }),
                    format!("{} {}", weapon_skill.value, weapon_skill.name.get(lang)),
                ]);
            }

            let mut table = table.build();
            table.with(Style::empty());
            out.push_str(table.to_string().as_str());
        }

        out.push_str("```");

        Self {
            description: out,
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_characters_to_status(lang: LocaleLang, characters: Vec<Character>) -> Self {
        let mut out = String::new();
        out.push_str("```text\n");

        let mut table = Builder::new();

        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Name).as_str(),
            "❤️",
            "🧠",
            "🍀",
            "🪄",
            locale_text_by_tag_lang(lang, LocaleTag::Dex).as_str(),
        ]);

        for character in characters.iter() {
            table.push_record([
                character.name.clone(),
                character.status_hp_raw(),
                character.status_sanity_raw(),
                character.status_luck_raw(),
                character.status_magic_raw(),
                character.attributes.dexterity().to_string(),
            ]);
        }
        let table = table.build().with(Style::empty()).to_string();
        out.push_str(table.as_str());
        out.push_str("```");

        Self {
            description: out,
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_to_sheet(lang: LocaleLang, character: &Character) -> Vec<Self> {
        let mut mcs = vec![Self::from_character_to_attributes(lang, character)];
        mcs.extend(Self::from_character_skills(lang, &character.skills));
        mcs.push(Self::from_character_to_equipment(lang, character));
        mcs
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_to_attributes(lang: LocaleLang, character: &Character) -> Self {
        let mut out = String::new();

        out.push_str(format!("**{}**\t{}\n", &character.name, &character.status_one_line()).as_str());

        if let Some(archetype) = &character.pulp_archetype {
            out.push_str(
                format!(
                    "{}: **{}**\n",
                    locale_text_by_tag_lang(lang, LocaleTag::PulpArchetype),
                    archetype.get(lang),
                )
                .as_str(),
            );
        }

        if !character.pulp_talents.is_empty() {
            out.push_str(
                format!(
                    "{}: {}\n",
                    locale_text_by_tag_lang(lang, LocaleTag::PulpTalents),
                    character
                        .pulp_talents
                        .iter()
                        .map(|t| format!("**{}**", t.get(lang)))
                        .join(", ")
                )
                .as_str(),
            );
        }

        out.push_str("```text\n");

        let mut table = Builder::new();

        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Str),
            character.attributes.strength().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Dex),
            character.attributes.dexterity().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Pow),
            character.attributes.power().to_string(),
        ]);

        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Con),
            character.attributes.constitution().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::App),
            character.attributes.appearance().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Edu),
            character.attributes.education().to_string(),
        ]);
        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Siz),
            character.attributes.size().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Int),
            character.attributes.intelligence().to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Move),
            character.move_rate.to_string(),
        ]);

        table.push_record([
            locale_text_by_tag_lang(lang, LocaleTag::Db),
            character.damage_modifier(lang),
            locale_text_by_tag_lang(lang, LocaleTag::Build),
            character.build.to_string(),
            locale_text_by_tag_lang(lang, LocaleTag::Dodge),
            character
                .get_skill(locale_text_by_tag_lang(LocaleLang::default(), LocaleTag::Dodge).as_str())
                .unwrap()
                .value
                .to_string(),
        ]);

        let mut table = table.build();
        table.with(Style::empty());

        let column_count = table.count_columns();
        for i in 0..column_count {
            let alignment = if i % 2 == 0 {
                Alignment::left()
            } else {
                Alignment::right()
            };
            table.with(Modify::new(Column::from(i)).with(alignment));
        }

        out.push_str(table.to_string().as_str());
        out.push_str("```\n");

        Self {
            description: out,
            ..Default::default()
        }
    }

    #[cfg(feature = "character-sheet")]
    pub fn from_character_to_equipment(lang: LocaleLang, character: &Character) -> Self {
        let mc_weapons = Self::from_character_to_weapons(lang, character);
        let mc_items = Self::from_character_to_items(lang, character);
        Self {
            description: format!("{}\n{}", mc_weapons.description, mc_items.description),
            ..Default::default()
        }
    }
}

impl From<Embed> for MessageContent {
    fn from(embed: Embed) -> Self {
        Self {
            title: embed.title.unwrap_or_default(),
            description: embed.description.unwrap_or_default(),
            footer: embed.footer.map_or_else(|| "".into(), |f| f.text),
            colour: embed.colour.map_or_else(|| None, |c| Some(c.0)),
        }
    }
}
