use crate::{
    character::{Ammo, RangeDamage, Weapon},
    commands::autocomplete::*,
    locale::{LocaleLang, LocaleTag, locale_text_by_tag_lang},
    message::MessageContent,
    types::*,
};
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    name_localized("pl", "gmbroń"),
    subcommands("add", "remove"),
    rename = "gmweapon"
)]
pub async fn gmweapon_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[poise::command(slash_command, name_localized("pl", "dodaj"))]
async fn add(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    character_name: String,
    #[name_localized("pl", "nazwa")] weapon_name: String,
    #[name_localized("pl", "obrażenia")] damage: String,
    #[name_localized("pl", "ataki")] attacks: String,
    #[name_localized("pl", "zasięg")] range: Option<String>,
    #[name_localized("pl", "zawodność")] malfunction: Option<i32>,
    #[name_localized("pl", "czy_modyfikator_obrażeń")] apply_damage_modifier: bool,
    #[name_localized("pl", "czy_połowa_modyfikatora_obrażeń")] half_damage_modifier: Option<bool>,
    #[name_localized("pl", "czy_ostra")] impaling: bool,
    #[name_localized("pl", "magazynek")] clip_capacity: Option<i32>,
    #[name_localized("pl", "amunicja_startowa")] start_ammunition: Option<i32>,
    #[name_localized("pl", "umiejętność")]
    #[autocomplete = "autocomplete_my_fight_skills"]
    skill: String,
) -> Result<(), Error> {
    let mut mc = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                let range_dmgs = RangeDamage::from(
                    range
                        .unwrap_or("-".into())
                        .replace(' ', "")
                        .replace('k', "d")
                        .split("/")
                        .map(str::to_string)
                        .collect(),
                    damage.replace(' ', "").split("/").map(str::to_string).collect(),
                )
                .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

                let ammo = if let (Some(clip_size), Some(ammunition)) = (clip_capacity, start_ammunition) {
                    Some(Ammo::new(clip_size, ammunition))
                } else {
                    None
                };

                let mut new_weapon = Weapon::new(
                    &weapon_name,
                    range_dmgs,
                    &attacks,
                    apply_damage_modifier,
                    impaling,
                    &skill,
                    malfunction,
                    ammo,
                );
                if half_damage_modifier.unwrap_or_default() {
                    new_weapon.apply_half_dm();
                }
                character.weapons.push(new_weapon);

                mc = Some(MessageContent {
                    title: format!("`{}`", character.name),
                    description: format!("➕ `{}`", weapon_name),
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

#[poise::command(slash_command, name_localized("pl", "usuń"))]
async fn remove(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[name_localized("pl", "imię")]
    character_name: String,
    #[autocomplete = "autocomplete_character_weapons"]
    #[name_localized("pl", "nazwa_przedmiotu")]
    weapon_name: String,
) -> Result<(), Error> {
    let mut mc = None;

    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            let user_lang = user_data.lang;
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                let position = character
                    .weapons
                    .iter()
                    .position(|n| n.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
                    .ok_or(format!(
                        "`{}` {}: `{}`",
                        character_name,
                        locale_text_by_tag_lang(user_lang, LocaleTag::NoSuchWeapon),
                        weapon_name
                    ))?;
                if character.weapons[position].default {
                    return Err(locale_text_by_tag_lang(user_lang, LocaleTag::CantRemoveDefaultWeapon).into());
                }
                character.weapons.remove(position);

                mc = Some(MessageContent {
                    title: format!("`{}`", character.name),
                    description: format!("➖ `{}`", weapon_name),
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
