use crate::{
    character::{Attributes, Character},
    commands::autocomplete::*,
    message::Message,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("create", "select", "delete", "reset"),
    rename = "character"
)]
pub async fn character_cmd(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[poise::command(prefix_command, slash_command)]
async fn create(
    ctx: Context<'_>,
    #[name_localized("pl", "imię")] name: String,
    #[name_localized("pl", "siła")] str: i32,
    #[name_localized("pl", "kondycja")] con: i32,
    #[name_localized("pl", "budowa_ciała")] siz: i32,
    #[name_localized("pl", "zręczność")] dex: i32,
    #[name_localized("pl", "wygląd")] app: i32,
    #[name_localized("pl", "inteligencja")] int: i32,
    #[name_localized("pl", "moc")] pow: i32,
    #[name_localized("pl", "wykształcenie")] edu: i32,
    #[name_localized("pl", "szczęście")] luck: i32,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let max = data.max_characters_per_user;
        let user_entry = data.users.entry(user_id).or_default();

        if user_entry.characters.len() >= max {
            message.title = "Sorry, you have too many characters already".to_string();
            ephemeral = true;
        } else {
            let attributes = Attributes::new(str, con, siz, dex, app, int, pow, edu)?;
            if user_entry
                .characters
                .insert(name.clone(), Character::new(&name, attributes, luck)?)
                .is_some()
            {
                message.title = format!("✅ `{name}` updated.")
            } else {
                message.title = format!("✅ `{name}` created.")
            }
            user_entry.active_character = Some(name);
        }
    }

    ctx.send(CreateReply::default().embed(message.to_embed()).ephemeral(ephemeral))
        .await?;

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;

        user_data.characters.remove(&name);

        if let Some(active_character) = &mut user_data.active_character {
            if active_character == &name {
                user_data.active_character = None;
            }
        }
    }

    ctx.send(
        CreateReply::default().embed(
            Message {
                title: format!("❎ `{name}` deleted."),
                ..Default::default()
            }
            .to_embed(),
        ),
    )
    .await?;

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn select(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_character"]
    #[name_localized("pl", "imię")]
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

        user_data.active_character = Some(character.name.clone());
    }

    ctx.send(
        CreateReply::default()
            .embed(
                Message {
                    title: format!("`{}` selected.", &name),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        character.reset();

        ctx.send(
            CreateReply::default().embed(
                Message {
                    title: format!("`{}` has been revived with full strength. 💪", &active),
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
