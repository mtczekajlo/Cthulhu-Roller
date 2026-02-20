use crate::{
    character::{Attributes, Character},
    commands::autocomplete::character::{
        autocomplete_my_character, autocomplete_my_pulp_talents, autocomplete_pulp_archetypes,
        autocomplete_pulp_talents,
    },
    locale::{LocaleTag, locale_entry_by_str, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    subcommands("create", "select", "remove", "reset", "pulp_talent"),
    rename = "character",
    aliases("postać")
)]
pub async fn character_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[poise::command(prefix_command, slash_command, aliases("stwórz"))]
async fn create(
    ctx: Context<'_>,
    #[name_localized("pl", "imię")] name: String,
    #[name_localized("pl", "zawód")] occupation: Option<String>,
    #[rename = "STR"]
    #[name_localized("pl", "S")]
    str: i32,
    #[rename = "CON"]
    #[name_localized("pl", "KON")]
    con: i32,
    #[rename = "SIZ"]
    #[name_localized("pl", "BC")]
    siz: i32,
    #[rename = "DEX"]
    #[name_localized("pl", "ZR")]
    dex: i32,
    #[rename = "APP"]
    #[name_localized("pl", "WYG")]
    app: i32,
    #[rename = "INT"] int: i32,
    #[rename = "POW"]
    #[name_localized("pl", "MOC")]
    pow: i32,
    #[rename = "EDU"]
    #[name_localized("pl", "WYK")]
    edu: i32,
    #[name_localized("pl", "szczęście")] luck: i32,
    #[autocomplete = "autocomplete_pulp_archetypes"]
    #[name_localized("pl", "pulpowy_archetyp")]
    pulp_archetype: Option<String>,
) -> Result<(), Error> {
    let mut mc = MessageContent::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let max = data.max_characters_per_user;
        let user_data = data.users.entry(user_id).or_default();

        if user_data.characters.len() >= max {
            mc.title = locale_text_by_tag_lang(user_data.lang, LocaleTag::SorryTooManyCharacters);
            ephemeral = true;
        } else {
            let attributes = Attributes::new(str, con, siz, dex, app, int, pow, edu)?;
            let pulp_archetype =
                pulp_archetype.map(|pulp_archetype| locale_entry_by_str(&pulp_archetype).unwrap().clone());
            user_data.characters.insert(
                name.clone(),
                Character::new(&name, &occupation, attributes, luck, pulp_archetype)?,
            );
            mc.title = format!("✅ `{name}`");
            user_data.active_character = Some(name);
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed()).ephemeral(ephemeral))
        .await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, aliases("usuń"))]
async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();

        user_data.characters.remove(&name);

        if let Some(active_character) = &mut user_data.active_character
            && active_character == &name
        {
            user_data.active_character = None;
        }
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

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, aliases("wybierz"))]
async fn select(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_character"]
    #[name_localized("pl", "imię")]
    name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let character = user_data.characters.get_mut(&name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &name
        ))?;

        user_data.active_character = Some(character.name.clone());
    }

    ctx.send(
        CreateReply::default()
            .embed(
                MessageContent {
                    title: format!("➡️ `{}`", &name),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, aliases("zresetuj"))]
async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    let mc;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let active = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&active).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &active
        ))?;

        character.reset();

        mc = MessageContent {
            title: format!("`{}`", &active),
            description: locale_text_by_tag_lang(user_data.lang, LocaleTag::ComesBackWithFullStrength),
            ..Default::default()
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(
    prefix_command,
    slash_command,
    aliases("pulp_talent"),
    subcommands("pulp_talent_add", "pulp_talent_remove")
)]
async fn pulp_talent(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("dodaj"), rename = "add")]
async fn pulp_talent_add(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_pulp_talents"]
    #[name_localized("pl", "pulpowy_talent")]
    pulp_talent: String,
) -> Result<(), Error> {
    let mc;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let active = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&active).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &active
        ))?;

        character
            .pulp_talents
            .push(locale_entry_by_str(&pulp_talent).unwrap().clone());

        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("✅ `{pulp_talent}`"),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, aliases("usuń"), rename = "remove")]
async fn pulp_talent_remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_pulp_talents"]
    #[name_localized("pl", "pulpowy_talent")]
    pulp_talent: String,
) -> Result<(), Error> {
    let mc;
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let active = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&active).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &active
        ))?;

        let position = character
            .pulp_talents
            .iter()
            .position(|lt| lt.partial_match_ignore_case(&pulp_talent))
            .ok_or("No such talent!")?;
        character.pulp_talents.remove(position);

        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("❌ `{pulp_talent}`"),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}
