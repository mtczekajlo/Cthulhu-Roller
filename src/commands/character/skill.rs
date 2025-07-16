use crate::autocomplete::*;
use crate::character::Character;
use crate::locale::*;
use crate::message::{format_improve, format_skill};
use crate::roll::{improve_skill, roll_impl, roll_skill, RealRng, SkillResult};
use crate::types::*;
use crate::{
    message::Message,
    roll::{ModifierDiceType, SuccessLevel},
};
use poise::CreateReply;
use regex::Regex;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("check", "add", "set", "delete", "improve", "mark"),
    rename = "skill"
)]
pub async fn skill_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn check(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let (skill_result, to_improve) = skill_impl(character, &name, &modifier_dice)?;

        let mut message = format_skill(&name, &skill_result, &user_data.lang)?;

        message.title = format!("{}\n{}", message.title, name);

        if to_improve {
            message.description = format!(
                "{}\n{}",
                message.description,
                locale_text_lang(&user_data.lang, &LocaleTag::SkillMarked)?
            );
        }

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn add(
    ctx: Context<'_>,
    #[name_localized("pl", "nazwa")] name: String,
    #[name_localized("pl", "wartość")] value: i32,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let skill_name = name.trim();

        character.add_skill(skill_name, value)?;

        let message = Message {
            title: format!("`{}`: ✅ `{}` = `{}`", character.name, skill_name, value),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn set(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] value: i32,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        character.set_skill(&name, value)?;

        let mut message = Message {
            title: format!("`{}`: ✅ `{}` = `{}`", character.name, name, value),
            ..Default::default()
        };

        if character.get_skill(&name).unwrap().name.equals("Cthulhu Mythos") {
            message.description = format!(
                "{} **{}**",
                locale_text_lang(&user_data.lang, &LocaleTag::MaxSanitySet)?,
                character.sanity.max
            );
        }

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        character.delete_skill(&name);

        let message = Message {
            title: format!("`{}`: ❌ `{}`", character.name, name),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

pub fn skill_impl(
    character: &mut Character,
    skill_name: &str,
    modifier_dice: &Option<String>,
) -> Result<(SkillResult, bool), Error> {
    let mut penalty = 0;
    let mut bonus = 0;
    if let Some(modifier_dice) = modifier_dice {
        let pattern = r"^([\+-]*)$";
        let re = Regex::new(pattern)?;
        let modifier_dice = modifier_dice.replace(' ', "");
        let captures = re
            .captures(&modifier_dice)
            .ok_or(format!("Invalid modifier dice: \"{modifier_dice}\""))?;
        match captures.get(1) {
            None => (),
            Some(captures_match) => {
                let penalty_bonus_str = captures_match.as_str();
                penalty = penalty_bonus_str.chars().filter(|c| *c == '-').count() as i32;
                bonus = penalty_bonus_str.chars().filter(|c| *c == '+').count() as i32;
            }
        }
    }

    let skill = character.get_mut_skill(skill_name).ok_or("No such skill")?;
    let skill_result = roll_skill(skill.value, penalty, bonus)?;

    let mut mark_to_improve = false;
    if skill.improvable && skill_result.success_level >= SuccessLevel::Success {
        match &skill_result.modifier_dice {
            None => mark_to_improve = true,
            Some(modifier_dice) => {
                if modifier_dice.dice_type == ModifierDiceType::Penalty {
                    mark_to_improve = true
                }
            }
        }
        skill.to_improve |= mark_to_improve;
    }

    Ok((skill_result, mark_to_improve))
}

#[poise::command(prefix_command, slash_command)]
async fn improve(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_improvable_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "kość_rozwoju")] improve_dice: Option<String>,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let skill = character.get_mut_skill(&name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_lang(&user_data.lang, &LocaleTag::NoSuchSkill).expect("Missing LOCALE_TEXTS entry"),
                name
            )
        })?;

        if skill.improvable && skill.to_improve {
            let improve_result = improve_skill(skill.value);
            let mut message = format_improve(name.clone(), &user_data.lang, improve_result.clone())?;
            skill.to_improve = false;

            let mut improve_dice = improve_dice.unwrap_or("d6".into()); //TODO Home Rules; rule-wise: d10
            if user_data.lang == LocaleLang::Polski {
                improve_dice = improve_dice.replace('d', "k");
            }
            if improve_result.success_level == SuccessLevel::Success {
                let res;
                {
                    let mut rng = RealRng::new();
                    res = roll_impl(&mut rng, &improve_dice)?;
                }
                let skill_value = skill.value;
                let skill_new_value = skill_value + res.result;
                character.set_skill(&name, skill_new_value)?;

                message.title = format!("{}\n{}", message.title, name);
                message.description = format!(
                    "{}\n{} + **{}** ({}) = **{}**",
                    message.description, skill_value, res.result, improve_dice, skill_new_value
                );
                ctx.send(CreateReply::default().embed(message.to_embed())).await?;
            }
        } else {
            let message = Message {
                title: format!(
                    "**{}** {}.",
                    name,
                    locale_text_lang(&user_data.lang, &LocaleTag::NotMarked)?
                ),
                ..Default::default()
            };

            ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(true))
                .await?;
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn mark(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
) -> Result<(), Error> {
    let message;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let skill = character.get_mut_skill(&name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_lang(&user_data.lang, &LocaleTag::NoSuchSkill).expect("Missing LOCALE_TEXTS entry"),
                name
            )
        })?;

        skill.to_improve = true;

        message = Message {
            title: format!("**{}**", name,),
            description: locale_text_lang(&user_data.lang, &LocaleTag::SkillMarked)?,
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    ctx.data().save().await?;
    Ok(())
}
