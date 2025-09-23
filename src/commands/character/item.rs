use crate::{
    character::Item,
    commands::autocomplete::*,
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "item",
    name_localized("pl", "przedmiot"),
    subcommands("list_cmd", "add_cmd", "give_cmd", "remove_cmd")
)]
pub async fn item_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, rename = "list", name_localized("pl", "lista"))]
async fn list_cmd(ctx: Context<'_>) -> Result<(), Error> {
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
        mc = MessageContent::from_character_items(user_data.lang, &character.items);
    }

    ctx.send(CreateReply::default().content(mc.to_content()).ephemeral(true))
        .await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(slash_command, rename = "add", name_localized("pl", "dodaj"))]
async fn add_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_items"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
    #[name_localized("pl", "ilość")] item_qty: Option<i32>,
    #[name_localized("pl", "jednostka")] item_unit: Option<String>,
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

        let item_qty = item_qty.unwrap_or(1);

        if let Some(position) = character
            .items
            .iter()
            .position(|n| n.name.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
        {
            let item = character.items.get_mut(position).unwrap();
            item.add(item_qty);
        } else {
            let item = Item::new(&item_name, item_qty, item_unit.as_deref());
            character.items.push(item);
        };

        message = MessageContent {
            title: format!("`{}`", character.name),
            description: format!(
                "➕ `{} [{} {}]`",
                item_name,
                item_qty,
                locale_text_by_tag_lang(user_data.lang, LocaleTag::Pcs)
            ),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(slash_command, rename = "remove", name_localized("pl", "usuń"))]
async fn remove_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_items"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
    #[autocomplete = "autocomplete_my_item_quantity"]
    #[name_localized("pl", "ilość")]
    item_qty: Option<String>,
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

        let item_qty = item_qty.unwrap_or("1".into()).parse()?;

        let position = character
            .items
            .iter()
            .position(|n| n.name.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
            .ok_or("No such item!")?;

        let item = character.items.get_mut(position).unwrap();

        item.remove(item_qty)
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        if item.quantity == 0 {
            character.items.remove(position);
        }

        message = MessageContent {
            title: format!("`{}`", character.name),
            description: format!(
                "➖ `{} [{} {}]`",
                item_name,
                item_qty,
                locale_text_by_tag_lang(user_data.lang, LocaleTag::Pcs)
            ),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(slash_command, rename = "give", name_localized("pl", "daj"))]
async fn give_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_items"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
    #[name_localized("pl", "ilość")] item_qty: Option<i32>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    character_name: String,
) -> Result<(), Error> {
    {
        let user_locale = ctx.locale().unwrap();
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let lang = if let Some(user_data) = data.users.get(&user_id) {
            user_data.lang
        } else {
            LocaleLang::from(user_locale)
        };
        let giver_name;
        let transferred_item;

        {
            if !data
                .users
                .iter()
                .any(|(_, user_data)| user_data.characters.iter().any(|(name, _)| **name == character_name))
            {
                let e = match lang {
                    LocaleLang::Polski => Err("Nieistniejący odbiorca!".into()),
                    LocaleLang::English => Err("No receiver!".into()),
                };
                return e;
            }
        }

        {
            let giver_data = data.users.entry(user_id).or_default();
            let active = giver_data.active_character.clone().ok_or("No active character.")?;
            let giver = giver_data
                .characters
                .get_mut(&active)
                .ok_or(format!("Character not found: `{}`", &active))?;
            giver_name = giver.name.clone();

            let position = giver
                .items
                .iter()
                .position(|n| n.name.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
                .ok_or("No such item!")?;

            let item = giver.items.get_mut(position).unwrap();

            transferred_item = item
                .remove(item_qty.unwrap_or(1))
                .map_err(|e| Error::from(e.to_string(lang)))?;

            if item.quantity == 0 {
                giver.items.remove(position);
            }
        }

        for taker_data in data.users.values_mut() {
            if let Some(taker) = taker_data.characters.get_mut(&character_name) {
                let message = MessageContent {
                    title: format!(
                        "`{}`: {} {} `{}` {} `{}`",
                        taker.name,
                        locale_text_by_tag_lang(lang, LocaleTag::Received),
                        locale_text_by_tag_lang(lang, LocaleTag::Item),
                        transferred_item.to_string(lang),
                        locale_text_by_tag_lang(lang, LocaleTag::From),
                        giver_name
                    ),
                    ..Default::default()
                };

                if let Some(position) = taker.items.iter().position(|n| {
                    n.name
                        .to_ascii_lowercase()
                        .contains(&transferred_item.name.to_ascii_lowercase())
                }) {
                    taker.items.get_mut(position).unwrap().add(transferred_item.quantity);
                } else {
                    taker.items.push(transferred_item);
                }

                ctx.send(CreateReply::default().embed(message.to_embed())).await?;
                break;
            }
        }
    }

    ctx.data().data.write().await.save().await
}
