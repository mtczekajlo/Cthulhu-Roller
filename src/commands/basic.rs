use crate::roller::croll::CrollResult;
use crate::roller::improve_roll::improve_skill;
use crate::{
    bot_data::*,
    commands::autocomplete::{autocomplete_attributes, autocomplete_battle},
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{
        battle::{Battle, CharacterInitiative},
        croll::croll,
        dice_rng::RealRng,
        roll::{roll_attributes, roll_query},
    },
    types::*,
};
use itertools::Itertools;
use poise::CreateReply;
use regex::Regex;

#[poise::command(prefix_command, slash_command, rename = "language", aliases("język"))]
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

#[poise::command(prefix_command, slash_command, rename = "croll")]
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

#[poise::command(prefix_command, slash_command, rename = "hcroll")]
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

#[poise::command(prefix_command, slash_command, rename = "improve_test", aliases("test_rozwoju"))]
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

#[poise::command(prefix_command, slash_command, rename = "roll")]
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
            roll_result = roll_query(&mut rng, &dice)?
        }

        message_content = MessageContent::from_dice_result(user_lang, roll_result, false);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "hroll")]
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
            roll_result = roll_query(&mut rng, &dice)?
        }

        message_content = MessageContent::from_dice_result(user_lang, roll_result, true);
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "initiative", aliases("inicjatywa"))]
pub async fn initiative_cmd(ctx: Context<'_>, #[name_localized("pl", "lista")] list: String) -> Result<(), Error> {
    let words: Vec<&str> = list.split_whitespace().collect();
    if !words.len().is_multiple_of(2) {
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
            if characters
                .iter()
                .map(|el| &el.name)
                .any(|el| el.eq_ignore_ascii_case(name))
            {
                return Err(format!("Non-unique character list: `{}`", list).into());
            }
            characters.push(CharacterInitiative {
                croll_result: skill_result,
                name: name.to_string(),
            });
        }
        battle = Battle::new(characters);

        message_content_a = MessageContent::from_battle(user_lang, &battle, true, None);
        message_content_b = MessageContent::from_battle(user_lang, &battle, false, None);
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

#[poise::command(prefix_command, slash_command, rename = "next_round", aliases("następna_runda"))]
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
            message_content = Some(MessageContent::from_battle(user_lang, battle, true, None));
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
    prefix_command,
    slash_command,
    rename = "previous_round",
    aliases("poprzednia_runda")
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
            message_content = Some(MessageContent::from_battle(user_lang, battle, true, None));
        }
    }

    if let Some(message) = message_content {
        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
        ctx.data().data.write().await.save().await?;
        return Ok(());
    }

    Err("No active battle".into())
}

#[poise::command(prefix_command, slash_command, rename = "end_battle", aliases("koniec_walki"))]
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

#[poise::command(prefix_command, slash_command, rename = "add_to_fight", aliases("dołącz_do_walki"))]
pub async fn add_to_fight_cmd(ctx: Context<'_>, list: String) -> Result<(), Error> {
    let words: Vec<&str> = list.split_whitespace().collect();
    if !words.len().is_multiple_of(2) {
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

    let message_content_a;
    let message_content_b;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        if let Some(battle) = &mut data.battle {
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
            battle.add_characters(&characters)?;

            let add_mes = Some(format!("**⚔️ {}**", characters.iter().map(|el| &el.name).join(", ")));

            message_content_a = MessageContent::from_battle(user_lang, battle, true, add_mes.clone());
            message_content_b = MessageContent::from_battle(user_lang, battle, false, add_mes.clone());
        } else {
            return Err("No active battle".into());
        }
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

#[poise::command(prefix_command, slash_command, rename = "remove_from_fight", aliases("usuń_z_walki"))]
pub async fn remove_from_fight_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_battle"] name: String,
) -> Result<(), Error> {
    let message_content;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get(&user_id);
        let user_lang = match user_data {
            Some(ud) => ud.lang,
            None => LocaleLang::default(),
        };

        if let Some(battle) = &mut data.battle {
            battle.remove_character(&name)?;

            message_content = MessageContent::from_battle(user_lang, battle, true, Some(format!("**💀 {}**", name)));
        } else {
            return Err("No active battle".into());
        }
    }

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;
    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "levels", aliases("poziomy"))]
pub async fn levels_cmd(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let threshold = threshold.parse::<i32>()?;
    let message_content = MessageContent::from_levels(threshold);
    ctx.send(CreateReply::default().embed(message_content.to_embed()).ephemeral(true))
        .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "roll_attributes", aliases("rzuć_cechy"))]
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

#[poise::command(prefix_command, slash_command, rename = "about", aliases("o_programie"))]
pub async fn about_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let message_content = MessageContent {
        title: format!("**{} {}**", std::env!("CARGO_PKG_NAME"), std::env!("CARGO_PKG_VERSION")),
        description: format!(
            "{}\n\n{}",
            std::env!("CARGO_PKG_DESCRIPTION"),
            std::env!("CARGO_PKG_REPOSITORY"),
        ),
        ..Default::default()
    };

    ctx.send(CreateReply::default().embed(message_content.to_embed()))
        .await?;

    Ok(())
}
