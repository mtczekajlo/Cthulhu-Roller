use crate::bot_data::*;
use crate::locale::*;
use crate::message::Message;
use crate::message::{format_dice, format_improve, format_initiative, format_levels, format_skill};
use crate::roll::{improve_skill, roll_impl, roll_skill, CharacterInitiative, InitiativeResult, RealRng, SkillResult};
use crate::types::*;
use poise::CreateReply;
use regex::Regex;

#[poise::command(slash_command)]
pub async fn language(ctx: Context<'_>, #[name_localized("pl", "język")] language: String) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let users = &mut data.users;
        if users.get_mut(&user_id).is_none() {
            users.insert(user_id, UserData::default());
        }
        let user_data = users.get_mut(&user_id).unwrap();

        user_data.lang = language.into();
        ctx.send(
            CreateReply::default().embed(
                Message {
                    title: format!("Set language to `{}`", user_data.lang),
                    ..Default::default()
                }
                .to_embed(),
            ),
        )
        .await?;
    }

    ctx.data().save().await?;
    Ok(())
}

pub fn croll_impl(query: &str) -> Result<SkillResult, Error> {
    let pattern = r"^(\d+)([\+-]*)$";
    let re = Regex::new(pattern)?;
    let query = query.replace(' ', "");
    let captures = re.captures(&query).ok_or(format!("Invalid query: \"{query}\""))?;
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
    roll_skill(threshold, penalty, bonus)
}
#[poise::command(slash_command, track_edits)]
pub async fn croll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    ctx.send(CreateReply::default().embed(format_skill(&threshold, &croll_impl(&threshold)?, &user_lang)?.to_embed()))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
pub async fn hcroll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    ctx.send(
        CreateReply::default()
            .ephemeral(true)
            .embed(format_skill(&threshold, &croll_impl(&threshold)?, &user_lang)?.to_embed()),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
pub async fn improve(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    let pattern = r"^\D*(\d+)\D*$";
    let re = Regex::new(pattern)?;
    let threshold_stripped = threshold.replace(' ', "");
    let captures = re
        .captures(&threshold_stripped)
        .ok_or(format!("Invalid query: \"{threshold_stripped}\""))?;
    let threshold_int = captures.get(1).ok_or("Invalid threshold:")?.as_str().parse()?;
    ctx.send(
        CreateReply::default()
            .embed(format_improve(threshold.clone(), &user_lang, improve_skill(threshold_int))?.to_embed()),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
pub async fn roll(ctx: Context<'_>, #[name_localized("pl", "kości")] dice: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    let roll_result;
    {
        let mut rng = RealRng::new();
        roll_result = roll_impl(&mut rng, &dice)?
    }

    ctx.send(CreateReply::default().embed(format_dice(dice.clone(), roll_result, &user_lang, true)?))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
pub async fn hroll(ctx: Context<'_>, #[name_localized("pl", "kości")] dice: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    let roll_result;
    {
        let mut rng = RealRng::new();
        roll_result = roll_impl(&mut rng, &dice)?
    }

    ctx.send(
        CreateReply::default()
            .embed(format_dice(dice.clone(), roll_result, &user_lang, true)?)
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
#[poise::command(slash_command, track_edits)]
pub async fn initiative(ctx: Context<'_>, #[name_localized("pl", "lista")] list: String) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id);
    let mut user_lang = LocaleLang::English;
    if let Some(user_data) = user_data {
        user_lang = user_data.lang.clone();
    }

    let words: Vec<&str> = list.split_whitespace().collect();
    if words.len() % 2 != 0 {
        ctx.send(
            CreateReply::default()
                .embed(
                    Message {
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

    let mut characters: Vec<CharacterInitiative> = vec![];
    for pair in words.chunks(2) {
        let name = pair[0];
        let threshold = pair[1];
        let skill_result = croll_impl(threshold)?;
        characters.push(CharacterInitiative {
            result: skill_result,
            name: name.to_string(),
        });
    }

    let initiative_result = InitiativeResult::new(characters);
    ctx.send(CreateReply::default().embed(format_initiative(
        list.clone(),
        initiative_result.clone(),
        &user_lang,
        false,
    )?))
    .await?;
    ctx.send(
        CreateReply::default()
            .embed(format_initiative(
                list.clone(),
                initiative_result.clone(),
                &user_lang,
                true,
            )?)
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
pub async fn levels(ctx: Context<'_>, #[name_localized("pl", "próg")] threshold: String) -> Result<(), Error> {
    let pattern = r"^\D*(\d+)\D*$";
    let re = Regex::new(pattern)?;
    let threshold_stripped = threshold.replace(' ', "");
    let captures = re
        .captures(&threshold_stripped)
        .ok_or(format!("Invalid query: \"{threshold_stripped}\""))?;
    let threshold_int = captures.get(1).ok_or("Invalid threshold:")?.as_str().parse()?;
    ctx.send(
        CreateReply::default()
            .embed(format_levels(threshold.clone(), threshold_int))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}
