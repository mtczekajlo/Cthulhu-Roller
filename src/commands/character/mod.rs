use crate::autocomplete::*;
use crate::bot_data::*;
use crate::character::Character;
use crate::commands::basic::croll_impl;
use crate::commands::character::skill::skill_impl;
use crate::locale::*;
use crate::message::format_skill;
use crate::message::format_skill_add_desc;
use crate::message::{format_sheet, format_skill_no_luck, Message};
use crate::roll::roll_skill;
use crate::roll::SuccessLevel;
use crate::types::*;
use poise::CreateReply;
pub mod character_cmd;
pub mod item_mod;
pub mod skill;
pub mod weapon;

#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id).ok_or("No characters.")?;
    let active = user_data.active_character.clone().ok_or("No active character.")?;
    let character = user_data
        .characters
        .get(&active)
        .ok_or(format!("Character not found: `{}`", &active))?;

    let message = Message {
        title: format!("`{}`", character.name),
        description: character.status(),
        ..Default::default()
    };

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(true))
        .await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn sheet(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id).ok_or("No characters.")?;
    let active = user_data.active_character.clone().ok_or("No active character.")?;
    let character = user_data
        .characters
        .get(&active)
        .ok_or(format!("Character not found: `{}`", &active))?;

    let messages = format_sheet(character, &user_data.lang)?;

    for message in messages {
        ctx.send(CreateReply::default().content(message).ephemeral(true))
            .await?;
    }

    Ok(())
}

pub fn hp_impl(character: &mut Character, delta: i32, lang: &LocaleLang) -> Result<Message, Error> {
    let mut message = Message::default();

    character.hp.modify(delta);

    if delta < -(character.hp.max) {
        character.dead = true;
        character.major_wound = false;
        message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::DeathInevitable)?);
    } else {
        if delta >= (character.hp.max / 2) && character.major_wound {
            character.major_wound = false;
            message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::MajorWoundHealed)?);
        }
        if delta <= -(character.hp.max / 2) {
            character.major_wound = true;
            message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::MajorWound)?);
            if character.hp.current > 0 {
                message.description = format!(
                    "{}\n{}\n{}",
                    message.description,
                    locale_text_lang(lang, &LocaleTag::YouFell)?,
                    locale_text_lang(lang, &LocaleTag::RollConCheckBlackOut)?
                )
                .to_string();
            }
        }

        if character.hp.current == 0 {
            if character.major_wound {
                message.description = format!(
                    "{}\n**{}**\n{}\n{}\n{}",
                    message.description,
                    locale_text_lang(lang, &LocaleTag::Agony)?,
                    locale_text_lang(lang, &LocaleTag::YouFell)?,
                    locale_text_lang(lang, &LocaleTag::YouBlackOut)?,
                    locale_text_lang(lang, &LocaleTag::RollConCheckDie)?
                );
            } else {
                message.description = format!(
                    "{}\n**{}**\n{}\n{}",
                    message.description,
                    locale_text_lang(lang, &LocaleTag::KnockOut)?,
                    locale_text_lang(lang, &LocaleTag::YouFell)?,
                    locale_text_lang(lang, &LocaleTag::YouBlackOut)?,
                );
            }
        }
    }

    message.title = format!("{} ❤️ **{:+}**\n{}", character.name(), delta, character.status_hp());

    Ok(message)
}

#[poise::command(slash_command)]
pub async fn hp(ctx: Context<'_>, #[name_localized("pl", "zmiana")] delta: Option<String>) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            message = hp_impl(character, delta, &user_data.lang)?;
        } else {
            message.title = character.status_hp();
            ephemeral = true;
        }
        ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(ephemeral))
            .await?;
    }

    if delta.is_some() {
        ctx.data().save().await?;
    }
    Ok(())
}

pub fn sanity_impl(character: &mut Character, delta: i32, lang: &LocaleLang) -> Result<Message, Error> {
    let mut message = Message::default();

    character.sanity.modify(delta);

    if character.sanity.current == 0 {
        character.insane = true;
        message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::MindShattered)?);
    } else if character.fragile_mind && delta < 0 {
        message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::TempInsanity)?);
    } else if delta <= -5 {
        message.description = format!(
            "**{}**\n{}",
            locale_text_lang(lang, &LocaleTag::TempInsanityThreat)?,
            locale_text_lang(lang, &LocaleTag::RollIntCheck)?
        );
    } else if (character.sanity.current as f32 / character.sanity.initial as f32) < 0.8 {
        character.fragile_mind = true;
        message.description = format!("**{}**", locale_text_lang(lang, &LocaleTag::IndefInsanity)?);
    }

    message.title = format!("{} 🧠 **{:+}**\n{}", character.name(), delta, character.status_sanity());

    Ok(message)
}

#[poise::command(slash_command)]
pub async fn sanity(ctx: Context<'_>, #[name_localized("pl", "zmiana")] delta: Option<String>) -> Result<(), Error> {
    let mut message;
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            message = sanity_impl(character, delta, &user_data.lang)?;
        } else {
            let sanity_str = locale_text_lang(&user_data.lang, &LocaleTag::Sanity)?;
            let sanity_result = croll_impl(&character.sanity.current.to_string())?;
            message = format_skill_no_luck(&sanity_str, &sanity_result, &user_data.lang)?;
            message.title = format!("{}\n{}", message.title, &sanity_str);
            ephemeral = true;
        }

        ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(ephemeral))
            .await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn luck(
    ctx: poise::Context<'_, Data, Error>,
    #[name_localized("pl", "zmiana")] delta: Option<String>,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;

            if delta < -character.luck.current {
                return Err(locale_text_lang(&user_data.lang, &LocaleTag::CantSpendLuck)?.into());
            }

            character.luck.modify(delta);

            message.title = format!(
                "🍀 {} **{:+}**\n{}",
                locale_text_lang(&user_data.lang, &LocaleTag::Luck)?,
                delta,
                character.status_luck()
            );
        } else {
            let luck_str = locale_text_lang(&user_data.lang, &LocaleTag::Luck)?;
            let luck_result = croll_impl(&character.luck.current.to_string())?;
            message = format_skill_no_luck(&luck_str, &luck_result, &user_data.lang)?;
            message.title = format!("{}\n{}", message.title, &luck_str);
            ephemeral = true;
        }
    }

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(ephemeral))
        .await?;

    if delta.is_some() {
        ctx.data().save().await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn magic(
    ctx: poise::Context<'_, Data, Error>,
    #[name_localized("pl", "zmiana")] delta: Option<String>,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            character.magic.modify(delta);

            message.title = format!(
                "🪄 {} **{:+}**\n{}",
                locale_text_lang(&user_data.lang, &LocaleTag::Mp)?,
                delta,
                character.status_magic()
            );
        } else {
            message.title = character.status_magic();
            ephemeral = true;
        }
    }

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(ephemeral))
        .await?;

    if delta.is_some() {
        ctx.data().save().await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn attribute(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_attributes"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] value: Option<i32>,
) -> Result<(), Error> {
    let mut message;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        if let Some(value) = value {
            character.set_attribute(&name, value);
            message = Message {
                title: format!("`{}` `{}` = `{}`", character.name, name, value),
                ..Default::default()
            };
        } else {
            message = format_skill(
                &name,
                &roll_skill(
                    character
                        .attributes
                        .get(&name)
                        .unwrap_or_else(|| panic!("No such attribute: {name}"))
                        .value,
                    0,
                    0,
                )?,
                &user_data.lang,
            )?;

            message.title = format!("**{}**\n{}", message.title, name);
        }

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}
#[poise::command(slash_command)]
pub async fn dodge(
    ctx: Context<'_>,
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

        let dodge_str = locale_text_lang(&user_data.lang, &LocaleTag::Dodge)?;
        let (dodge_result, to_improve) = skill_impl(character, &dodge_str, &modifier_dice)?;

        let mut message = format_skill(&dodge_str, &dodge_result, &user_data.lang)?;
        message.title = format!("{}\n{}", message.title, dodge_str);

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

#[poise::command(slash_command)]
pub async fn fight(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
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

        let weapon = character
            .weapons
            .iter()
            .find(|&w| w.name.partial_match(&weapon_name.to_ascii_lowercase()))
            .cloned()
            .ok_or("No such weapon")?;

        let (mut skill_result, mut to_improve) = skill_impl(character, &weapon.skill, &modifier_dice)?;

        let mut add_desc = None;
        if let Some(malfunction) = weapon.malfunction {
            if skill_result.result >= malfunction {
                skill_result.success_level = SuccessLevel::CriticalFailure;
                to_improve = false;
                add_desc = Some(locale_text_lang(&user_data.lang, &LocaleTag::WeaponJammed)?);
            }
        }

        let mut message = format_skill_add_desc(&weapon_name, &skill_result, &user_data.lang, add_desc.as_deref())?;

        message.title = format!(
            "{}\n{}\n{}",
            message.title,
            weapon_name,
            character
                .get_skill(&weapon.skill)
                .ok_or("No such skill")?
                .name
                .get(&user_data.lang)
        );

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

#[poise::command(slash_command)]
pub async fn sleep(ctx: Context<'_>) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        character.sanity.update_initial();

        ctx.send(
            CreateReply::default().embed(
                Message {
                    title: format!("`{}` has survived another day. 🎉", character.name),
                    description: format!(
                        "Updating initial 🧠 Sanity: **{}**/{}",
                        character.sanity.current, character.sanity.initial
                    ),
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

//TODO: refactor /fight and /shoot to select from weapons instead of skills
//TODO: add /maneuver
