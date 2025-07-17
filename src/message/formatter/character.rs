use crate::{
    Error,
    character::{Character, Skill},
    locale::{LocaleLang, LocaleTag, locale_text_lang},
    message::{Message, format_skill_impl},
    roller::SkillResult,
};
use itertools::{
    EitherOrBoth::{Both, Left, Right},
    Itertools,
};
use tabled::{
    builder::Builder,
    settings::{Alignment, Modify, Style, object::Column},
};

pub fn format_skill_no_luck(query: &str, roll_result: &SkillResult, lang: &LocaleLang) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, false, None)
}

pub fn format_skill_add_desc(
    query: &str,
    roll_result: &SkillResult,
    lang: &LocaleLang,
    additional_desc: Option<&str>,
) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, true, additional_desc)
}

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

pub fn format_sheet(character: &Character, lang: &LocaleLang) -> Result<Vec<String>, Error> {
    Ok(vec![
        format_attributes(character, lang)?,
        format_skills(character, lang)?,
        format_equipment(character, lang)?,
    ])
}

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

pub fn format_equipment(character: &Character, lang: &LocaleLang) -> Result<String, Error> {
    let mut out = format_weapons(character, lang)?;
    out.push_str(format_items(character, lang)?.as_str());
    Ok(out)
}

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
