pub mod attribute;
pub mod character_cmd;
pub mod fight;
pub mod interaction;
pub mod item;
pub mod skill;
pub mod skills;
pub mod stats;
pub mod weapon;

use crate::{
    locale::{LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(slash_command, rename = "status", name_localized("pl", "status"))]
pub async fn status_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mc;
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

        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: character.status(),
            ..Default::default()
        };
    }
    ctx.send(CreateReply::default().embed(mc.to_embed()).ephemeral(true))
        .await?;
    Ok(())
}

#[poise::command(slash_command, rename = "sheet", name_localized("pl", "karta"))]
pub async fn sheet_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mcs;
    {
        let user_id = ctx.author().id.get();
        let data = ctx.data().data.read().await;
        let user_data = data.users.get(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data
            .characters
            .get(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        mcs = MessageContent::from_character_to_sheet(user_data.lang, character);
    }

    for mc in mcs {
        ctx.send(CreateReply::default().content(mc.to_content()).ephemeral(true))
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command, rename = "sleep", name_localized("pl", "śpij"))]
pub async fn sleep_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mc;
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

        character.sanity.update_initial();

        mc = MessageContent {
            title: format!("`{}` has survived another day. 🎉", character.name),
            description: format!(
                "Updating initial 🧠 Sanity: **{}**/{}",
                character.sanity.current, character.sanity.initial
            ),
            ..Default::default()
        };
    }
    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}
