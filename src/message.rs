use crate::roll::{DiceResult, ImproveResult, InitiativeResult, SkillResult, SuccessLevel};
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};

#[derive(Default)]
pub struct RollMessage {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub colour: Option<u32>,
}

impl RollMessage {
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

pub fn format_skill(query: String, roll_result: SkillResult) -> CreateEmbed {
    let mut message = RollMessage::default();

    let mut rolls_str = roll_result
        .ten_rolls
        .iter()
        .fold(String::new(), |s, el| format!("{s} [ {el}0 ]"));
    rolls_str = format!(
        "{rolls_str} [ {one_roll} ]",
        one_roll = roll_result.one_roll
    );
    message.title = format!("**{}**", roll_result.success_level);
    message.colour = Some(roll_result.success_level.hex());
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
    message.to_embed()
}

pub fn format_dice(query: String, roll_result: DiceResult) -> CreateEmbed {
    let mut message = RollMessage::default();

    let title = format!("**{}**", roll_result.result);

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
    description = format!("Rolls: {description}");
    message.title = title;
    message.description = description;
    message.footer = format!("Query: \"{query}\"");
    message.to_embed()
}

pub fn format_initiative(query: String, roll_result: InitiativeResult) -> CreateEmbed {
    let mut message = RollMessage::default();

    let title = "Initiative".to_string();

    let description = roll_result
        .characters
        .iter()
        .enumerate()
        .fold("".to_string(), |s, v| format!("{}\n{}. {}", s, v.0, v.1));
    message.title = title;
    message.description = description;
    message.footer = format!("Query: \"{query}\"");
    message.to_embed()
}

pub fn format_improve(query: String, improve_result: ImproveResult) -> CreateEmbed {
    let message = RollMessage {
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
