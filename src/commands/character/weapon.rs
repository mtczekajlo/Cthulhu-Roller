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
    rename = "weapon",
    name_localized("pl", "broń"),
    subcommands(
        "list_cmd",
        "add_cmd",
        "give_cmd",
        "remove_cmd",
        "reload_cmd",
        "add_ammo_cmd",
        "remove_ammo_cmd"
    )
)]
pub async fn weapon_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "list", name_localized("pl", "lista"))]
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
        mc = MessageContent::from_character_to_weapons(user_data.lang, character);
    }

    ctx.send(CreateReply::default().content(mc.to_content()).ephemeral(true))
        .await?;
    ctx.data().data.write().await.save().await
}

#[allow(clippy::too_many_arguments)]
#[poise::command(prefix_command, slash_command, rename = "add", name_localized("pl", "dodaj"))]
async fn add_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "nazwa")] name: String,
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
            &name,
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

        message = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("➕ **{}**", name),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(message.to_embed())).await?;

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "remove", name_localized("pl", "usuń"))]
async fn remove_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
) -> Result<(), Error> {
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

        let position = character
            .weapons
            .iter()
            .position(|n| n.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
            .ok_or("No such weapon!")?;
        character.weapons.remove(position);

        let mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("➖ **{}**", weapon_name,),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    }

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(slash_command, rename = "give", name_localized("pl", "daj"))]
async fn give_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
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
        let weapon;

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
                .weapons
                .iter()
                .position(|w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()) && !w.default)
                .ok_or("No such weapon!")?;

            weapon = giver.weapons.remove(position);
        }

        for receiver_data in data.users.values_mut() {
            if let Some(receiver) = receiver_data.characters.get_mut(&character_name) {
                let message = MessageContent {
                    title: format!(
                        "`{}`: {} {} `{}` {} `{}`",
                        receiver.name,
                        locale_text_by_tag_lang(lang, LocaleTag::Received),
                        locale_text_by_tag_lang(lang, LocaleTag::Weapon),
                        weapon.name,
                        locale_text_by_tag_lang(lang, LocaleTag::From),
                        giver_name
                    ),
                    ..Default::default()
                };

                receiver.weapons.push(weapon);

                ctx.send(CreateReply::default().embed(message.to_embed())).await?;
                break;
            }
        }
    }

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "reload", name_localized("pl", "przeładuj"))]
pub async fn reload_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
) -> Result<(), Error> {
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

        let weapon = character
            .weapons
            .iter_mut()
            .find(|w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
            .ok_or("No such weapon")?;

        let desc = match weapon.reload() {
            Ok(o) => o.to_string(user_data.lang),
            Err(e) => e.to_string(user_data.lang),
        };

        let message = MessageContent {
            title: format!("**{}** (`{}`)", weapon.name, &character_name),
            description: desc,
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "add_ammo",
    name_localized("pl", "dodaj_amunicję")
)]
pub async fn add_ammo_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
    #[name_localized("pl", "amunicja")] rounds: i32,
) -> Result<(), Error> {
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

        let weapon = character
            .weapons
            .iter_mut()
            .find(|w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
            .ok_or("No such weapon")?;

        let desc = match weapon.add_ammo(rounds) {
            Ok(o) => o.to_string(user_data.lang),
            Err(e) => e.to_string(user_data.lang),
        };

        let message = MessageContent {
            title: format!("**{}** (`{}`)", weapon.name, &character_name),
            description: desc,
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "remove_ammo",
    name_localized("pl", "usuń_amunicję")
)]
pub async fn remove_ammo_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
    #[name_localized("pl", "amunicja")] rounds: i32,
) -> Result<(), Error> {
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

        let weapon = character
            .weapons
            .iter_mut()
            .find(|w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
            .ok_or("No such weapon")?;

        let desc = match weapon.remove_ammo(rounds) {
            Ok(o) => o.to_string(user_data.lang),
            Err(e) => e.to_string(user_data.lang),
        };

        let message = MessageContent {
            title: format!("**{}** (`{}`)", weapon.name, &character_name),
            description: desc,
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().data.write().await.save().await?;

    Ok(())
}
