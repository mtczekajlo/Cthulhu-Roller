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
    rename = "gmitem",
    aliases("gmprzedmiot"),
    subcommands("add_cmd", "remove_cmd")
)]
pub async fn gmitem_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "add", aliases("dodaj"))]
async fn add_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    character_name: String,
    #[autocomplete = "autocomplete_character_items"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
    #[name_localized("pl", "ilość")] item_qty: Option<i32>,
    #[name_localized("pl", "jednostka")] item_unit: Option<String>,
) -> Result<(), Error> {
    let mut mc = None;
    {
        let item_qty = item_qty.unwrap_or(1);

        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                let item_unit = if let Some(position) = character
                    .items
                    .iter()
                    .position(|n| n.name.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
                {
                    let item = character.items.get_mut(position).unwrap();
                    item.add(item_qty);
                    item.unit
                        .clone()
                        .unwrap_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::Pcs))
                } else {
                    let item = Item::new(&item_name, item_qty, item_unit.as_deref());
                    character.items.push(item);
                    item_unit.unwrap_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::Pcs))
                };

                mc = Some(MessageContent {
                    title: format!("`{}`", character.name),
                    description: format!("➕ `{} [{} {}]`", item_name, item_qty, item_unit),
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
            "`{}` {}",
            character_name,
            locale_text_by_tag_lang(LocaleLang::default(), LocaleTag::CharacterNotFound)
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "remove", aliases("usuń"))]
async fn remove_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    character_name: String,
    #[autocomplete = "autocomplete_character_items"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    item_name: String,
    #[name_localized("pl", "ilość")] item_qty: Option<i32>,
) -> Result<(), Error> {
    let mut mc = None;

    {
        let item_qty = item_qty.unwrap_or(1);

        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            let user_lang = user_data.lang;
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                let position = character
                    .items
                    .iter()
                    .position(|n| n.name.to_ascii_lowercase().contains(&item_name.to_ascii_lowercase()))
                    .ok_or(format!(
                        "`{}` {}: `{}`",
                        character_name,
                        locale_text_by_tag_lang(user_lang, LocaleTag::NoSuchItem),
                        item_name
                    ))?;

                let item = character.items.get_mut(position).unwrap();

                let removed_item = item
                    .remove(item_qty)
                    .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

                if item.quantity == 0 {
                    character.items.remove(position);
                }

                mc = Some(MessageContent {
                    title: format!("`{}`", character.name),
                    description: format!(
                        "➖ `{} [{} {}]`",
                        item_name,
                        removed_item.quantity,
                        removed_item
                            .unit
                            .unwrap_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::Pcs))
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
            locale_text_by_tag_lang(LocaleLang::default(), LocaleTag::CharacterNotFound),
            character_name,
        )
        .into());
    }

    ctx.data().data.write().await.save().await
}
