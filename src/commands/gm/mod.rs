use crate::autocomplete::*;
use crate::bot_data::*;
use crate::character::Character;
use crate::commands::character::hp_impl;
use crate::commands::character::sanity_impl;
use crate::locale::*;
use crate::message::{format_sheet, Message};
use crate::types::*;
use poise::CreateReply;
pub mod gmcharacter;
use crate::message::format_gmstatus;
use poise::serenity_prelude::{Attachment, CreateAttachment};

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

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn gmsheet(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let data = ctx.data().data.read().await;

        for user_data in data.users.values() {
            if let Some(character) = user_data.characters.get(&name) {
                let messages = format_sheet(character, &user_data.lang)?;

                for message in messages {
                    ctx.send(CreateReply::default().content(message).ephemeral(true))
                        .await?;
                }

                break;
            }
        }
    }

    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn gmstatus(ctx: Context<'_>) -> Result<(), Error> {
    let mut message = Message {
        title: "Characters' status".to_string(),
        ..Default::default()
    };

    let data = ctx.data().data.read().await.clone();

    if data.users.is_empty() {
        message.description = "No active characters.".to_string();
    } else {
        let mut characters: Vec<Character> = vec![];
        for (_, user_data) in data.users.clone().into_iter() {
            if let Some(active) = &user_data.active_character {
                if let Some(character) = user_data.characters.get(active) {
                    characters.push(character.clone());
                }
            }
        }
        characters.sort();

        message.description = format_gmstatus(characters);
    }

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(true))
        .await?;

    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn insane(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.fragile_mind = true;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "**`{}`{}**",
                                character.name,
                                locale_text_lang(&user_data.lang, &LocaleTag::GoneMad)?
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn sane(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.fragile_mind = false;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "**`{}`, {}**",
                                character.name,
                                locale_text_lang(&user_data.lang, &LocaleTag::MindHealed)?
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn heal(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.major_wound = false;
                character.dead = false;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "**`{}`, {}**",
                                character.name,
                                locale_text_lang(&user_data.lang, &LocaleTag::BodyHealed)?
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn kill(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.hp.current = 0;
                character.dead = true;
                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("**`{}` has died.**", &name),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn revive(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.dead = false;
                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("**`{}` has been revived!**", &name),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn gmhp(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
    #[name_localized("pl", "zmiana")] delta: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                let delta = delta.replace(' ', "").parse::<i32>()?;
                let message = hp_impl(character, delta, &user_data.lang)?;

                ctx.send(CreateReply::default().embed(message.to_embed())).await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn gmsanity(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    name: String,
    #[name_localized("pl", "zmiana")] delta: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                let delta = delta.replace(' ', "").parse::<i32>()?;
                let message = sanity_impl(character, delta, &user_data.lang)?;

                ctx.send(CreateReply::default().embed(message.to_embed())).await?;
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn quicksave(ctx: Context<'_>) -> Result<(), Error> {
    {
        ctx.data().quicksave().await?;
    }
    ctx.send(
        CreateReply::default()
            .embed(
                Message {
                    description: "DB quicksave".to_string(),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn quickload(ctx: Context<'_>) -> Result<(), Error> {
    {
        ctx.data().quickload().await?;
    }
    ctx.send(
        CreateReply::default()
            .embed(
                Message {
                    description: "DB quickload".to_string(),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, check = "is_user_gm")]
pub async fn db(
    ctx: poise::Context<'_, Data, Error>,
    #[name_localized("pl", "plik")] file: Option<Attachment>,
) -> Result<(), Error> {
    if let Some(file) = file {
        let response = reqwest::get(&file.url).await?;
        let content = response.text().await?;
        let db_parsed = serde_json::from_str::<InnerData>(&content)?;

        {
            let mut inner_data;
            inner_data = ctx.data().data.write().await;
            inner_data.clone_from(&db_parsed);
        }

        ctx.data().save().await?;

        ctx.send(
            CreateReply::default()
                .embed(
                    Message {
                        description: "DB saved.".to_string(),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
    } else {
        let file_attachment = CreateAttachment::path(ctx.data().get_db_path()).await?;
        ctx.send(
            CreateReply::default()
                .content("📦 Here’s your current database backup:")
                .attachment(file_attachment)
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
