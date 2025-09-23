use crate::{
    commands::{autocomplete::*, gm::is_user_gm},
    locale::{LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmcharacter",
    aliases("gmpostać"),
    subcommands("remove_cmd", "reset_cmd")
)]
pub async fn gmcharacter_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "remove",
    aliases("usuń"),
    check = "is_user_gm"
)]
async fn remove_cmd(
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

                if let Some(active_character) = &mut user_data.active_character
                    && active_character == &name
                {
                    user_data.active_character = None;
                }

                ctx.send(
                    CreateReply::default().embed(
                        MessageContent {
                            title: format!("❌ `{name}`"),
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

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "reset",
    aliases("zresetuj"),
    check = "is_user_gm"
)]
async fn reset_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            let user_lang = user_data.lang;
            if let Some(character) = user_data.characters.get_mut(&name) {
                character.reset();

                ctx.send(
                    CreateReply::default().embed(
                        MessageContent {
                            title: format!(
                                "`{}` {}",
                                &character.name,
                                locale_text_by_tag_lang(user_lang, LocaleTag::ComesBackWithFullStrength)
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

    ctx.data().data.write().await.save().await
}
