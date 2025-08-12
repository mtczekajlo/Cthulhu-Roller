use crate::roller::croll::CrollResult;
use crate::roller::improve_roll::improve_skill;
use crate::{
    bot_data::*,
    commands::autocomplete::autocomplete_attributes,
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{
        battle::{Battle, CharacterInitiative},
        croll::croll,
        dice_rng::RealRng,
        roll::{roll_attributes, roll_impl},
    },
    types::*,
};
use poise::CreateReply;
use regex::Regex;

#[poise::command(slash_command, rename = "language", name_localized("pl", "język"))]
pub async fn language_cmd(ctx: Context<'_>, #[name_localized("pl", "język")] language: String) -> Result<(), Error> {
    let message_content;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let users = &mut data.users;
        if users.get_mut(&user_id).is_none() {
            users.insert(user_id, UserData::default());
        }
        let user_data = users.get_mut(&user_id).unwrap();
        user_data.lang = language.into();

        message_content = MessageContent {
            title: format!(
                "{} `{}`",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::SetLanguageTo),
                user_data.lang
            ),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()).ephemeral(true))
        .await?;

    ctx.data().data.write().await.save().await
}

pub fn croll_impl(query: &str) -> Result<CrollResult, Error> {
    let pattern = r"^(\d+)([\+-]*)$";
    let re = Regex::new(pattern)?;
    let q = query.replace(' ', "");
    let captures = re.captures(&q).ok_or(format!("Invalid query: \"{q}\""))?;
    let threshold = captures.get(1).ok_or("Invalid threshold:")?.as_str().parse()?;
    let mut penalty = 0;
    let mut bonus = 0;
    match captures.get(2) {
        None => (),
        Some(captures_match) => {
            let penalty_bonus_str = captures_match.as_str();
            penalty = penalty_bonus_str.chars().filter(|c| *c == '-').count() as i32;
            bonus = penalty_bonus_str.chars().filter(|c| *c == '+').count() as i32;
        }
    }
    croll(query, threshold, penalty, bonus)
}

#[poise::command(slash_command, track_edits, rename = "croll")]
pub async fn croll_cmd(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let message_content;

    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        let croll_result = croll_impl(&threshold)?;

        message_content = MessageContent::from_croll_result(user_lang, &croll_result, false, false);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits, rename = "hcroll")]
pub async fn hcroll_cmd(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let message_content;

    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        let croll_result = croll_impl(&threshold)?;

        message_content = MessageContent::from_croll_result(user_lang, &croll_result, true, true);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    rename = "improve_test",
    name_localized("pl", "test_rozwoju"),
    track_edits
)]
pub async fn improve_test_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "próg")] threshold: String,
) -> Result<(), Error> {
    let message_content;
    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let mut user_lang = LocaleLang::default();
        if let Some(user_data) = user_data {
            user_lang = user_data.lang;
        }

        let threshold = threshold.parse::<i32>()?;
        let improve_result = improve_skill(threshold);
        message_content = MessageContent::from_improve(user_lang, &improve_result);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits, rename = "roll")]
pub async fn roll_cmd(ctx: Context<'_>, #[name_localized("pl", "kości")] dice: String) -> Result<(), Error> {
    let message_content;

    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        let roll_result;
        {
            let mut rng = RealRng::new();
            roll_result = roll_impl(&mut rng, &dice)?
        }

        message_content = MessageContent::from_dice_result(user_lang, roll_result, false);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits, rename = "hroll")]
pub async fn hroll_cmd(ctx: Context<'_>, #[name_localized("pl", "kości")] dice: String) -> Result<(), Error> {
    let message_content;

    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        let roll_result;
        {
            let mut rng = RealRng::new();
            roll_result = roll_impl(&mut rng, &dice)?
        }

        message_content = MessageContent::from_dice_result(user_lang, roll_result, true);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    rename = "initiative",
    name_localized("pl", "inicjatywa"),
    track_edits
)]
pub async fn initiative_cmd(ctx: Context<'_>, #[name_localized("pl", "lista")] list: String) -> Result<(), Error> {
    let words: Vec<&str> = list.split_whitespace().collect();
    if words.len() % 2 != 0 {
        ctx.send(
            CreateReply::default()
                .embed(
                    MessageContent {
                        description: format!("Query must contain pairs of `Name` and `Dex` thresholds: \"{list}\""),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let battle;
    let message_content_a;
    let message_content_b;
    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        let mut characters: Vec<CharacterInitiative> = vec![];
        for pair in words.chunks(2) {
            let name = pair[0];
            let threshold = pair[1];
            let skill_result = croll_impl(threshold)?;
            characters.push(CharacterInitiative {
                croll_result: skill_result,
                name: name.to_string(),
            });
        }
        battle = Battle::new(characters);

        message_content_a = MessageContent::from_battle(user_lang, &battle, true);
        message_content_b = MessageContent::from_battle(user_lang, &battle, false);
    }
    {
        let mut data = ctx.data().data.write().await;
        data.battle = Some(battle);
    }

    ctx.send(CreateReply::default().embed(message_content_a.to_embed()))
        .await?;
    ctx.send(
        CreateReply::default()
            .embed(message_content_b.to_embed())
            .ephemeral(true),
    )
    .await?;
    ctx.data().data.write().await.save().await
}

#[poise::command(
    slash_command,
    rename = "next_round",
    name_localized("pl", "następna_runda"),
    track_edits
)]
pub async fn next_round_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mut message_content = None;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        if let Some(battle) = &mut data.battle {
            battle.next_round();
            message_content = Some(MessageContent::from_battle(user_lang, battle, true));
        }
    }

    if let Some(message) = message_content {
        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
        ctx.data().data.write().await.save().await?;
        return Ok(());
    }

    Err("No active battle".into())
}

#[poise::command(
    slash_command,
    rename = "previous_round",
    name_localized("pl", "poprzednia_runda"),
    track_edits
)]
pub async fn previous_round_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mut message_content = None;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        if let Some(battle) = &mut data.battle {
            battle.previous_round();
            message_content = Some(MessageContent::from_battle(user_lang, battle, true));
        }
    }

    if let Some(message) = message_content {
        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
        ctx.data().data.write().await.save().await?;
        return Ok(());
    }

    Err("No active battle".into())
}

#[poise::command(
    slash_command,
    rename = "end_battle",
    name_localized("pl", "koniec_walki"),
    track_edits
)]
pub async fn end_battle_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let message_content;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        if data.battle.is_none() {
            return Err("No active battle".into());
        }

        data.battle = None;

        message_content = MessageContent {
            title: locale_text_by_tag_lang(user_lang, LocaleTag::FightEnd),
            ..Default::default()
        }
        .to_embed();
    }

    ctx.send(CreateReply::default().embed(message_content)).await?;
    ctx.data().data.write().await.save().await
}

#[poise::command(slash_command, rename = "levels", name_localized("pl", "poziomy"), track_edits)]
pub async fn levels_cmd(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let threshold = threshold.parse::<i32>()?;
    let message_content = MessageContent::from_levels(threshold);
    ctx.send(CreateReply::default().embed(message_content.to_embed()).ephemeral(true))
        .await?;
    Ok(())
}

#[poise::command(slash_command, rename = "roll_attributes", name_localized("pl", "rzuć_cechy"))]
pub async fn roll_attributes_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_attributes"]
    #[name_localized("pl", "pulpowa_cecha_podstawowa")]
    pulp_core_attribute: Option<String>,
    #[name_localized("pl", "przerzucaj_niska_suma")] reroll_low_sum: Option<bool>,
    #[name_localized("pl", "suma_szybkich_zasad")] quick_rules_sum: Option<bool>,
    #[name_localized("pl", "min_wartość_cechy")] min_attribute_value: Option<i32>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let user_lang = match user_data {
        Some(ud) => ud.lang,
        None => LocaleLang::default(),
    };

    let mut attribute_roll_result;

    loop {
        attribute_roll_result = roll_attributes(pulp_core_attribute.as_deref());

        if quick_rules_sum.unwrap_or_default() && !attribute_roll_result.is_sum_eq_quick_rules() {
            continue;
        }
        if reroll_low_sum.unwrap_or_default() && attribute_roll_result.is_sum_lt_quick_rules() {
            continue;
        }
        if attribute_roll_result.lowest_attribute_value() < min_attribute_value.unwrap_or_default() {
            continue;
        }
        break;
    }

    let message_content = MessageContent::from_attributes_result(user_lang, attribute_roll_result);

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;

    Ok(())
}
