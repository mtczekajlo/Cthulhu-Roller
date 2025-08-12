use crate::{
    commands::autocomplete::*,
    locale::{LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "add",
    name_localized("pl", "dodaj"),
    subcommands("basic_cmd", "specialized_cmd",)
)]
pub async fn add_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "basic", name_localized("pl", "podstawowa"))]
async fn basic_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_additional_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] value: i32,
) -> Result<(), Error> {
    let message;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let character_name = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&character_name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &character_name
        ))?;

        let skill_name = name.trim();

        character
            .add_skill(skill_name, value)
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        message = MessageContent {
            title: format!("`{}`: ✅ `{}` = `{}`", character.name, skill_name, value),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "specialized",
    name_localized("pl", "specjalizacja")
)]
async fn specialized_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_specialized_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "specjalizacja")] specialization: String,
    #[name_localized("pl", "wartość")] value: i32,
) -> Result<(), Error> {
    let message;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let character_name = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&character_name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &character_name
        ))?;

        let skill_name = format!("{} ({})", name.trim(), specialization);

        character
            .add_skill(&skill_name, value)
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        message = MessageContent {
            title: format!("`{}`: ✅ `{}` = `{}`", character.name, skill_name, value),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;

    ctx.data().data.write().await.save().await?;

    Ok(())
}
