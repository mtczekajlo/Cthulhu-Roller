use crate::{
    commands::{autocomplete::*, gm::is_user_gm},
    message::Message,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("delete", "reset"),
    rename = "gmcharacter"
)]
pub async fn gmcharacter_cmd(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello there!").await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, check = "is_user_gm")]
async fn delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if user_data.characters.get_mut(&name).is_some() {
                user_data.characters.remove(&name);
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
                break;
            }
        }
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, check = "is_user_gm")]
async fn reset(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.reset();

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("`{}` has been revived with full strength. 💪", &character.name),
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
