use crate::roll::{DiceResult, ImproveResult, InitiativeResult, SkillResult, SuccessLevel};
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
            None => Colour::from(SuccessLevel::Success.hex()),
            Some(colour) => Colour::from(colour),
        });
        embed
    }
}

pub fn format_skill(query: String, roll_result: SkillResult, full_output: bool) -> CreateEmbed {
    let mut message = Message {
        title: format!("**{}**", roll_result.success_level),
        colour: Some(roll_result.success_level.hex()),
        ..Default::default()
    };

    if full_output {
        let mut rolls_str = roll_result
            .ten_rolls
            .iter()
            .fold(String::new(), |s, el| format!("{s} [ {el}0 ]"));
        rolls_str = format!(
            "{rolls_str} [ {one_roll} ]",
            one_roll = roll_result.one_roll
        );
        let mut description = format!("**{}**", roll_result.result);
        let mut footer = String::new();
        let threshold = roll_result.threshold;
        let hard_threshold = threshold / 2;
        let extreme_threshold = threshold / 5;
        if roll_result.result > threshold {
            footer = format!("{} points to Success", roll_result.result - threshold);
        } else if roll_result.result > hard_threshold {
            footer = format!(
                "{} points to Hard Success",
                roll_result.result - hard_threshold
            );
        } else if roll_result.result > extreme_threshold {
            footer = format!(
                "{} points to Extreme Success",
                roll_result.result - extreme_threshold
            );
        }
        footer = format!(
            "{}\nThreshold: {} / {} / {}",
            footer, threshold, hard_threshold, extreme_threshold
        );
        if roll_result.penalty != 0 {
            footer = format!("{}\nPenalty dice: {}", footer, roll_result.penalty);
        }
        if roll_result.bonus != 0 {
            footer = format!("{}\nBonus dice: {}", footer, roll_result.bonus);
        }
        footer = format!("{}\nQuery: \"{}\"", footer, query);
        description = format!("{}\nRolls: {}", description, rolls_str);

        message.description = description;
        message.footer = footer;
    }

    message.to_embed()
}

pub fn format_dice(query: String, roll_result: DiceResult, full_output: bool) -> CreateEmbed {
    let mut message = Message {
        title: format!("**{}**", roll_result.result),
        ..Default::default()
    };

    if full_output {
        let mut description = roll_result
            .rolls
            .iter()
            .fold("".to_string(), |s, v| format!("{s} [ {v} ]"));
        if let Some(multiplier) = roll_result.multiplier {
            description = format!("( {description} ) x {multiplier}");
        }
        if let Some(modifier) = roll_result.modifier {
            description = format!(
                "{description} {sign} {modifier}",
                sign = if modifier > 0 { "+" } else { "-" },
                modifier = modifier.abs()
            );
        }
        message.description = format!("Rolls: {description}");
        message.footer = format!("Query: \"{query}\"");
    }

    message.to_embed()
}

pub fn format_initiative(
    query: String,
    roll_result: InitiativeResult,
    full_output: bool,
) -> CreateEmbed {
    let mut message = Message {
        title: "Initiative order:".to_string(),
        ..Default::default()
    };

    message.description = format_initiative_table(roll_result, full_output);
    if full_output {
        message.footer = format!("Query: \"{query}\"");
    }
    message.to_embed()
}

fn format_initiative_table(roll_result: InitiativeResult, full_output: bool) -> String {
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
        let result = format!("{}{}", character.result.success_level, result);
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
    let table = table.build().with(Style::psql()).to_string();
    out.push_str(table.as_str());
    out.push_str("```");
    out
}

pub fn format_improve(query: String, improve_result: ImproveResult) -> CreateEmbed {
    let message = Message {
        title: format!("**{}**", improve_result.success_level),
        colour: Some(improve_result.success_level.hex()),
        description: format!("**{}**", improve_result.result),
        footer: format!(
            "Threshold: {}\nQuery: \"{}\"",
            improve_result.threshold, query
        ),
    };
    message.to_embed()
}

pub fn format_levels(query: String, threshold: i32) -> CreateEmbed {
    let message = Message {
        title: format!("**{} / {} / {}**", threshold, threshold / 2, threshold / 5),
        colour: None,
        description: String::new(),
        footer: format!("Threshold: {}\nQuery: \"{}\"", threshold, query),
    };
    message.to_embed()
}
