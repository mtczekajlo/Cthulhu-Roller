#[cfg(feature = "character-sheet")]
use crate::character::{Character, Skill};
#[cfg(feature = "character-sheet")]
use itertools::{
    EitherOrBoth::{Both, Left, Right},
    Itertools,
};

#[cfg(feature = "character-sheet")]
use tabled::settings::{object::Column, Alignment, Modify};

use crate::{
    locale::{locale_text_lang, LocaleLang, LocaleTag},
    roll::{DiceResult, ImproveResult, InitiativeResult, SkillResult, SuccessLevel},
    Error,
};
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};
use tabled::{builder::Builder, settings::Style};

#[derive(Default)]
pub struct Message {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub colour: Option<u32>,
}

impl Message {
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
}

#[cfg(feature = "character-sheet")]
pub fn format_skill_no_luck(query: &str, roll_result: &SkillResult, lang: &LocaleLang) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, false, None)
}

pub fn format_skill(query: &str, roll_result: &SkillResult, lang: &LocaleLang) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, true, None)
}

#[cfg(feature = "character-sheet")]
pub fn format_skill_add_desc(
    query: &str,
    roll_result: &SkillResult,
    lang: &LocaleLang,
    additional_desc: Option<&str>,
) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, true, additional_desc)
}

pub fn format_skill_impl(
    query: &str,
    roll_result: &SkillResult,
    lang: &LocaleLang,
    show_thresholds: bool,
    additional_desc: Option<&str>,
) -> Result<Message, Error> {
    let mut message = Message {
        title: format!(
            "**{}**",
            locale_text_lang(lang, &roll_result.success_level.to_locale_tag())?,
        ),
        colour: Some(roll_result.success_level.hex()),
        ..Default::default()
    };

    let mut rolls_str = roll_result
        .ten_rolls
        .iter()
        .fold(String::new(), |s, el| format!("{s} `[{el}0]`"));

    rolls_str = format!("{rolls_str} `[{one_roll}]`", one_roll = roll_result.one_roll);

    let mut description = format!("**{}** / {}\n", roll_result.result, roll_result.threshold);

    let mut footer = String::new();

    if show_thresholds {
        let success_level = roll_result.success_level;
        for higher_skill_result in success_level {
            footer = format!(
                "{}{} {}: {}\n",
                footer,
                higher_skill_result.delta(roll_result.result, roll_result.threshold),
                locale_text_lang(lang, &LocaleTag::PointsTo)?,
                locale_text_lang(lang, &higher_skill_result.to_locale_tag())?,
            );
        }
    }

    if let Some(modifier_dice) = &roll_result.modifier_dice {
        footer = format!(
            "{}{} {}: {}\n",
            footer,
            locale_text_lang(lang, &modifier_dice.dice_type.to_locale_tag())?,
            locale_text_lang(lang, &LocaleTag::Dice)?,
            modifier_dice.count
        );
    }

    footer = format!("{footer}query: {query}");

    description = format!(
        "{}{}: {}",
        description,
        locale_text_lang(lang, &LocaleTag::Rolls)?,
        rolls_str
    );

    if let Some(add_desc) = additional_desc {
        description = format!("{}\n{}", description, add_desc);
    }

    message.description = description;
    message.footer = footer;

    Ok(message)
}

pub fn format_dice(
    query: String,
    roll_result: DiceResult,
    lang: &LocaleLang,
    full_output: bool,
) -> Result<CreateEmbed, Error> {
    let mut message = Message {
        title: format!("**{}**", roll_result.result),
        ..Default::default()
    };

    if full_output {
        message.description = format!(
            "{}:\n{}",
            locale_text_lang(lang, &LocaleTag::Rolls)?,
            roll_result.roll_msg
        );
        message.footer = format!("query: \"{query}\"");
    }

    if lang == &LocaleLang::Polski {
        message.description = message.description.replace("d", "k");
    }

    Ok(message.to_embed())
}

pub fn format_initiative(
    query: String,
    roll_result: InitiativeResult,
    lang: &LocaleLang,
    full_output: bool,
) -> Result<CreateEmbed, Error> {
    let mut message = Message {
        title: "Initiative Order:".to_string(),
        ..Default::default()
    };

    message.description = format_initiative_table(roll_result, lang, full_output)?;
    if full_output {
        message.footer = format!("query: \"{query}\"");
    }
    Ok(message.to_embed())
}

fn format_initiative_table(
    roll_result: InitiativeResult,
    lang: &LocaleLang,
    full_output: bool,
) -> Result<String, Error> {
    let mut out = String::new();
    out.push_str("```text\n");

    let mut table = Builder::new();

    if full_output {
        table.push_record(["#", "Name", "Result", "Dex", "Roll"]);
    } else {
        table.push_record(["#", "Name", "Result"]);
    }

    for (i, character) in roll_result.characters.iter().enumerate() {
        let i = i + 1;
        let result = match character.result.success_level {
            SuccessLevel::CriticalSuccess => "\n^ bonus die to first action",
            SuccessLevel::CriticalFailure => "\n^ loses first turn",
            _ => "",
        };
        let result = format!(
            "{}{}",
            locale_text_lang(lang, &character.result.success_level.to_locale_tag())?,
            result
        );
        if full_output {
            table.push_record([
                (i).to_string(),
                character.name.clone(),
                result,
                character.result.threshold.to_string(),
                character.result.result.to_string(),
            ]);
        } else {
            table.push_record([(i).to_string(), character.name.clone(), result]);
        }
    }
    let table = table.build().with(Style::empty()).to_string();
    out.push_str(table.as_str());
    out.push_str("```");
    Ok(out)
}

#[cfg(feature = "character-sheet")]
pub fn format_gmstatus(characters: Vec<Character>) -> String {
    let mut out = String::new();
    out.push_str("```text\n");

    let mut table = Builder::new();

    table.push_record(["Name", "❤️", "🧠", "🍀", "🪄"]);

    for character in characters.iter() {
        table.push_record([
            character.name_raw(),
            character.status_hp_raw(),
            character.status_sanity_raw(),
            character.status_luck_raw(),
            character.status_magic_raw(),
        ]);
    }
    let table = table.build().with(Style::empty()).to_string();
    out.push_str(table.as_str());
    out.push_str("```");
    out
}

#[cfg(feature = "character-sheet")]
pub fn format_sheet(character: &Character, lang: &LocaleLang) -> Result<Vec<String>, Error> {
    Ok(vec![
        format_attributes(character, lang)?,
        format_skills(character, lang)?,
        format_equipment(character, lang)?,
    ])
}

#[cfg(feature = "character-sheet")]
fn format_attributes(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = String::new();

    out.push_str(format!("**{}**\t{}\n", &character.name, &character.status_one_line()).as_str());

    out.push_str("```text\n");

    let mut table = Builder::new();

    table.push_record([
        locale_text_lang(lang, &LocaleTag::Str)?,
        character.attributes.strength.value.to_string(),
        locale_text_lang(lang, &LocaleTag::Dex)?,
        character.attributes.dexterity.value.to_string(),
        locale_text_lang(lang, &LocaleTag::Pow)?,
        character.attributes.power.value.to_string(),
    ]);

    table.push_record([
        locale_text_lang(lang, &LocaleTag::Con)?,
        character.attributes.constitution.value.to_string(),
        locale_text_lang(lang, &LocaleTag::App)?,
        character.attributes.appearance.value.to_string(),
        locale_text_lang(lang, &LocaleTag::Edu)?,
        character.attributes.education.value.to_string(),
    ]);
    table.push_record([
        locale_text_lang(lang, &LocaleTag::Siz)?,
        character.attributes.size.value.to_string(),
        locale_text_lang(lang, &LocaleTag::Int)?,
        character.attributes.intelligence.value.to_string(),
        locale_text_lang(lang, &LocaleTag::Move)?,
        character.move_rate.to_string(),
    ]);

    table.push_record([
        locale_text_lang(lang, &LocaleTag::Db)?,
        character.damage_modifier(lang),
        locale_text_lang(lang, &LocaleTag::Build)?,
        character.build.to_string(),
        locale_text_lang(lang, &LocaleTag::Dodge)?,
        character
            .get_skill("Dodge")
            .ok_or("No such skill: Dodge")?
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

    Ok(out)
}

#[cfg(feature = "character-sheet")]
fn format_skills(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = String::new();

    let mut skill_list: Vec<(String, Skill)> = character
        .skills
        .clone()
        .into_iter()
        .map(|el| (el.1.name.get(lang), el.1))
        .collect();
    skill_list.sort();

    let columns = 2;
    let records_per_column = skill_list.len() / columns + skill_list.len() % columns;
    let tail = skill_list.split_off(records_per_column);

    out.push_str("```text\n");

    let mut table = Builder::new();

    for pair in skill_list.iter().zip_longest(tail.iter()) {
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
        (1, Alignment::left()),  // 2nd column
        (2, Alignment::right()), // 3rd column
        (4, Alignment::left()),  // 5th column
        (5, Alignment::right()), // 6th column
    ];

    for (index, align) in alignments {
        table.with(Modify::new(Column::from(index)).with(align));
    }

    out.push_str(table.to_string().as_str());
    out.push_str("```\n");

    Ok(out)
}

#[cfg(feature = "character-sheet")]
pub fn format_equipment(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = format_weapons(character, lang)?;
    out.push_str(format_items(character, lang)?.as_str());
    Ok(out)
}

#[cfg(feature = "character-sheet")]
pub fn format_items(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = String::new();

    let mut eq_list: Vec<String> = character.items.clone();
    eq_list.sort();

    out.push_str(format!("{}\n```text\n", locale_text_lang(lang, &LocaleTag::Items)?).as_str());

    if eq_list.is_empty() {
        out.push_str(format!("({})\n", locale_text_lang(lang, &LocaleTag::NoItems)?).as_str());
    } else {
        let mut table = Builder::new();

        let columns = 2;
        let records_per_column = eq_list.len() / columns + eq_list.len() % columns;
        let tail = eq_list.split_off(records_per_column);

        for pair in eq_list.iter().zip_longest(tail.iter()) {
            match pair {
                Both(left, right) => table.push_record([
                    &("- ".to_string() + left.as_str()),
                    &("- ".to_string() + right.as_str()),
                ]),

                Left(left) => table.push_record([&("- ".to_string() + left.as_str()), ""]),

                Right(right) => table.push_record(["", &("- ".to_string() + right.as_str())]),
            }
        }

        let mut table = table.build();
        table.with(Style::empty());
        out.push_str(table.to_string().as_str());
    }
    out.push_str("```\n");

    Ok(out)
}

#[cfg(feature = "character-sheet")]
pub fn format_weapons(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = String::new();

    let mut weapon_list: Vec<_> = character.weapons.clone();
    weapon_list.sort();

    out.push_str(format!("{}\n```text\n", locale_text_lang(lang, &LocaleTag::Weapons)?).as_str());

    if weapon_list.is_empty() {
        out.push_str(format!("({})\n", locale_text_lang(lang, &LocaleTag::NoItems)?).as_str());
    } else {
        let mut table = Builder::new();

        table.push_record([
            locale_text_lang(lang, &LocaleTag::Weapon)?,
            locale_text_lang(lang, &LocaleTag::Damage)?,
            locale_text_lang(lang, &LocaleTag::Range)?,
            locale_text_lang(lang, &LocaleTag::Malfunction)?,
            locale_text_lang(lang, &LocaleTag::Skill)?,
        ]);

        for weapon in weapon_list {
            let mut weapon_dmg = weapon.dmg;
            match lang {
                LocaleLang::English => weapon_dmg = weapon_dmg.replace('k', "d"),
                LocaleLang::Polski => weapon_dmg = weapon_dmg.replace('d', "k"),
            }
            let dmg = format!(
                "{}{}{}",
                weapon_dmg,
                if weapon.apply_damage_modifier && character.build != 0 {
                    format!(
                        "+{}{} ({}{})",
                        if weapon.half_damage_modifier { "½" } else { "" },
                        locale_text_lang(lang, &LocaleTag::Db)?,
                        character.damage_modifier(lang),
                        if weapon.half_damage_modifier { "x½" } else { "" },
                    )
                } else {
                    "".to_string()
                },
                if weapon.impaling {
                    format!(" ({})", locale_text_lang(lang, &LocaleTag::Impaling)?)
                } else {
                    "".to_string()
                },
            );
            table.push_record([
                weapon.name.get(lang),
                dmg,
                weapon.range.map_or("-".to_string(), |n| format!("{} m", n)),
                weapon.malfunction.map_or("-".to_string(), |n| n.to_string()),
                character
                    .get_skill(&weapon.skill)
                    .ok_or("No such skill")?
                    .name
                    .get(lang),
            ]);
        }

        let mut table = table.build();
        table.with(Style::empty());
        out.push_str(table.to_string().as_str());
    }

    out.push_str("```\n");

    Ok(out)
}

pub fn format_improve(query: String, lang: &LocaleLang, improve_result: ImproveResult) -> Result<Message, Error> {
    let message = Message {
        title: format!(
            "**{}**",
            locale_text_lang(lang, &improve_result.success_level.to_locale_tag())?,
        ),
        colour: Some(improve_result.success_level.hex()),
        description: format!("**{}** / {}", improve_result.result, improve_result.threshold),
        footer: format!("query: {query}"),
    };
    Ok(message)
}

pub fn format_levels(query: String, threshold: i32) -> CreateEmbed {
    let message = Message {
        title: format!("**{} / {} / {}**", threshold, threshold / 2, threshold / 5),
        colour: None,
        description: "".into(),
        footer: format!("query: {query}"),
    };
    message.to_embed()
}
