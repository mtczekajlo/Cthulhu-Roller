use crate::{
    character::Character,
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{croll::CrollResult, success_level::SuccessLevel},
    types::Error,
};
use poise::{
    ReplyHandle,
    serenity_prelude::{self as serenity, CreateButton, ShardMessenger},
};
use std::time::Duration;

pub fn ok_button() -> CreateButton {
    serenity::CreateButton::new("OK")
        .label("OK")
        .style(serenity::ButtonStyle::Primary)
}

pub fn add_push_roll_button(croll_result: &CrollResult, lang: LocaleLang) -> Vec<CreateButton> {
    let mut buttons: Vec<CreateButton> = vec![];
    if croll_result.success_level < SuccessLevel::Success && croll_result.success_level != SuccessLevel::CriticalFailure
    {
        let button = serenity::CreateButton::new(locale_text_by_tag_lang(lang, LocaleTag::PushRoll))
            .label(locale_text_by_tag_lang(lang, LocaleTag::PushRoll))
            .style(serenity::ButtonStyle::Primary);
        buttons.push(button);
    }
    buttons
}

pub fn add_spend_luck_buttons(
    croll_result: &CrollResult,
    character: &Character,
    lang: LocaleLang,
) -> Vec<CreateButton> {
    let mut buttons: Vec<CreateButton> = vec![];
    let mut previous_delta = 0;
    for success_level in croll_result.success_level {
        let luck_delta = success_level.delta(croll_result.result(), croll_result.threshold);
        if luck_delta < 0 || previous_delta == luck_delta {
            break;
        }
        previous_delta = luck_delta;
        let mut button = serenity::CreateButton::new(success_level.to_string_lang(lang))
            .label(format!(
                "🍀{}➡️{}",
                luck_delta,
                locale_text_by_tag_lang(lang, success_level.to_locale_tag())
            ))
            .style(serenity::ButtonStyle::Primary);
        if luck_delta > character.luck.current {
            button = button.disabled(true);
        }
        buttons.push(button);
    }
    buttons
}

pub async fn handle_interaction(shard: ShardMessenger, reply: &ReplyHandle<'_>) -> Result<Option<String>, Error> {
    let orig_message = reply.message().await?.into_owned();
    let embed = orig_message.embeds[0].clone();
    let mut mc = MessageContent::from(embed);
    let mut ir = None;

    if let Some(interaction) = serenity::ComponentInteractionCollector::new(shard)
        .channel_id(orig_message.channel_id)
        .message_id(orig_message.id)
        .timeout(Duration::from_secs(3 * 60))
        .await
    {
        ir = match interaction.data.custom_id.as_str() {
            "OK" => None,
            s => Some(s.to_string()),
        };

        if let Some(ir) = &ir {
            mc.description = format!("{}\n\n➡️ {}", mc.description, ir);
        }
    }

    Ok(ir)
}
