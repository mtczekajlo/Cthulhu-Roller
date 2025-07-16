use crate::autocomplete::*;
use crate::{
    message::{format_items, Message},
    types::*,
};
use poise::CreateReply;

#[poise::command(prefix_command, slash_command, subcommands("list", "add", "delete"))]
pub async fn item(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        ctx.send(CreateReply::default().content(format_items(character, &user_data.lang)?))
            .await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn add(ctx: Context<'_>, #[name_localized("pl", "nazwa_przedmiotu")] item_name: String) -> Result<(), Error> {
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

        character.items.push(item_name.clone());
        message = Message {
            title: format!("{}\n+ `{}`", character.name(), item_name),
            ..Default::default()
        };
        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_equipment"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
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

        let position = character
            .items
            .iter()
            .position(|n| n.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
            .ok_or("No such item!")?;
        character.items.remove(position);

        message = Message {
            title: format!("{}\n- `{}`", character.name(), item_name),
            ..Default::default()
        };
        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}
