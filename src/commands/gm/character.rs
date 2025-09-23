use crate::commands::character::stats::{hp_impl, san_impl};
use crate::commands::gm::character::character::autocomplete_any_active_character;
use crate::{character::Character, commands::autocomplete::*, locale::*, message::MessageContent, types::*};
use poise::CreateReply;

pub async fn is_user_gm(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let gm_role_id;
        {
            let data = ctx.data().data.read().await;
            let guild = guild_id
                .to_guild_cached(ctx.serenity_context())
                .ok_or("Guild not in cache")?;

            gm_role_id = guild
                .role_by_name(&data.gm_role_name)
                .ok_or(format!("No such role `{}`", &data.gm_role_name))?
                .id
                .get();
        }
        if let Ok(member) = guild_id.member(ctx.serenity_context(), ctx.author().id).await {
            let has_role = member.roles.iter().any(|r| r.get() == gm_role_id);
            return Ok(has_role);
        }
    }

    Ok(false)
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmsheet",
    aliases("gmkarta"),
    check = "is_user_gm"
)]
pub async fn gmsheet_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mcs = None;

    {
        let data = ctx.data().data.read().await;

        for user_data in data.users.values() {
            if let Some(character) = user_data.characters.get(&name) {
                mcs = Some(MessageContent::from_character_to_sheet(gm_lang, character));
                break;
            }
        }
    }

    if let Some(mcs) = mcs {
        for mc in mcs {
            ctx.send(CreateReply::default().content(mc.to_content()).ephemeral(true))
                .await?;
        }
    } else {
        return Err(locale_text_by_tag_lang(gm_lang, LocaleTag::NoCharacters)
            .to_string()
            .into());
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmstatus",
    aliases("gmstatus"),
    check = "is_user_gm"
)]
pub async fn gmstatus_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let data = ctx.data().data.read().await.clone();

    let message = if data.users.is_empty() {
        return Err(locale_text_by_tag_lang(gm_lang, LocaleTag::NoCharacterSelected).into());
    } else {
        let mut characters: Vec<Character> = vec![];
        for (_, user_data) in data.users.clone().into_iter() {
            if let Some(active) = &user_data.active_character
                && let Some(character) = user_data.characters.get(active)
            {
                characters.push(character.clone());
            }
        }
        characters.sort();

        MessageContent::from_characters_to_status(gm_lang, characters)
    };

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(true))
        .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gminsane",
    aliases("gmoszalej"),
    check = "is_user_gm"
)]
pub async fn gminsane_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.fragile_mind = true;

                mc = Some(MessageContent {
                    title: format!(
                        "**`{}` {}**",
                        character.name,
                        locale_text_by_tag_lang(gm_lang, LocaleTag::GoneMad)
                    ),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmsane",
    aliases("gmotrzeźwiej"),
    check = "is_user_gm"
)]
pub async fn gmsane_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.fragile_mind = false;

                mc = Some(MessageContent {
                    title: format!(
                        "**`{}`, {}**",
                        character.name,
                        locale_text_by_tag_lang(gm_lang, LocaleTag::MindHealed)
                    ),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmwound",
    aliases("gmzrań"),
    check = "is_user_gm"
)]
pub async fn gmwound_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.major_wound = true;

                mc = Some(MessageContent {
                    title: format!(
                        "**`{}`, {}**",
                        character.name,
                        locale_text_by_tag_lang(gm_lang, LocaleTag::BodyWounded)
                    ),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmheal",
    aliases("gmulecz"),
    check = "is_user_gm"
)]
pub async fn gmheal_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.major_wound = false;

                mc = Some(MessageContent {
                    title: format!(
                        "**`{}`, {}**",
                        character.name,
                        locale_text_by_tag_lang(gm_lang, LocaleTag::BodyHealed)
                    ),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmkill",
    aliases("gmzabij"),
    check = "is_user_gm"
)]
pub async fn gmkill_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.hp.current = 0;
                character.dead = true;
                mc = Some(MessageContent {
                    title: format!("**`{}` 💀**", &name),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmrevive",
    aliases("gmwskrześ"),
    check = "is_user_gm"
)]
pub async fn gmrevive_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                character.dead = false;
                mc = Some(MessageContent {
                    title: format!("**`{}` 🩵**", &name),
                    ..Default::default()
                });
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "gmhp", aliases("gmpw"))]
pub async fn gmhp_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
    #[name_localized("pl", "zmiana")] delta: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                mc = Some(hp_impl(character, &delta, gm_lang)?);
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "gmsan", aliases("gmpp"))]
pub async fn gmsan_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
    #[name_localized("pl", "zmiana")] delta: String,
) -> Result<(), Error> {
    let gm_lang;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        gm_lang = user_data.lang;
    }

    let mut mc = None;
    let mut character_name = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character_name = Some(character.name.clone());
                mc = Some(san_impl(character, &delta, gm_lang)?);
                break;
            }
        }
    }

    if let Some(mc) = mc {
        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    } else {
        return Err(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(gm_lang, LocaleTag::CharacterNotFound),
            character_name.unwrap(),
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}
