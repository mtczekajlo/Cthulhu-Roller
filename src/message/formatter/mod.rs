use crate::{
    Error,
    locale::{LocaleLang, LocaleTag, locale_text_lang},
    message::Message,
    roller::{DiceResult, ImproveResult, InitiativeResult, SkillResult, SuccessLevel},
};
use poise::serenity_prelude::CreateEmbed;
use tabled::{builder::Builder, settings::Style};
#[cfg(feature = "character-sheet")]
pub mod character;
#[cfg(feature = "character-sheet")]
pub use character::*;

pub fn format_skill(query: &str, roll_result: &SkillResult, lang: &LocaleLang) -> Result<Message, Error> {
    format_skill_impl(query, roll_result, lang, true, None)
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
