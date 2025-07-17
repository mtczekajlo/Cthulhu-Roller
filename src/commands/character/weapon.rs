use crate::{
    character::Weapon,
    commands::autocomplete::*,
    locale::locale_text_lang,
    message::{Message, format_weapons},
    roller::{RealRng, get_roll_max, merge_dice_results, roll_impl},
    types::*,
};
use itertools::{Itertools, fold};
use poise::CreateReply;

#[poise::command(prefix_command, slash_command, subcommands("list", "add", "delete", "damage"))]
pub async fn weapon(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
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

        ctx.send(CreateReply::default().content(format_weapons(character, &user_data.lang)?))
            .await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[poise::command(prefix_command, slash_command)]
async fn add(
    ctx: Context<'_>,
    #[name_localized("pl", "nazwa")] name: String,
    #[name_localized("pl", "obrażenia")] damage: String,
    #[name_localized("pl", "zasięg")] range: Option<String>,
    #[name_localized("pl", "zawodność")] malfunction: Option<i32>,
    #[name_localized("pl", "czy_mo")] apply_damage_modifier: bool,
    #[name_localized("pl", "czy_połowa_mo")] half_damage_modifier: Option<bool>,
    #[name_localized("pl", "czy_ostra")] impaling: bool,
    #[name_localized("pl", "umiejętność")]
    #[autocomplete = "autocomplete_my_fight_skills"]
    skill: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let mut ranges: Vec<Option<i32>> = vec![];
        if let Some(range) = range {
            ranges = range.replace(' ', "").split("/").map(|r| r.parse().ok()).collect();
        }

        let dmgs: Vec<String> = damage.replace(' ', "").split("/").map(str::to_string).collect();

        if dmgs.len() == 1 {
            let mut new_weapon = Weapon::new(
                &name,
                &dmgs[0],
                apply_damage_modifier,
                impaling,
                &skill,
                None,
                malfunction,
            );
            if half_damage_modifier.unwrap_or_default() {
                new_weapon.apply_half_dm();
            }
            character.weapons.push(new_weapon);
        } else {
            let mut new_weapons = vec![];
            for pair in dmgs.iter().zip_longest(ranges) {
                match pair {
                    itertools::EitherOrBoth::Both(dmg, range) => {
                        let name = format!("{} ({} m)", name, range.unwrap());
                        let mut new_weapon =
                            Weapon::new(&name, dmg, apply_damage_modifier, impaling, &skill, range, malfunction);
                        if half_damage_modifier.unwrap_or_default() {
                            new_weapon.apply_half_dm();
                        }
                        new_weapons.push(new_weapon);
                    }
                    _ => return Err("Not equal number of damages and ranges!".into()),
                }
            }
            character.weapons.extend(new_weapons);
        }

        let message = Message {
            title: format!("`{}`: ✅ `{}`", character.name, name),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn delete(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
) -> Result<(), Error> {
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
            .weapons
            .iter()
            .position(|n| n.name.partial_match(&weapon_name))
            .ok_or("No such weapon!")?;
        character.weapons.remove(position);

        let message = Message {
            title: format!("`{}`: ❌ `{}`", character.name, weapon_name),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn damage(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
    #[name_localized("pl", "ekstremalne_obrażenia")] extreme_damage: Option<bool>,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data.active_character.clone().ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character not found: `{}`", &active))?;

        let weapon = character
            .weapons
            .iter()
            .find(|&w| w.name.partial_match(&weapon_name.to_ascii_lowercase()))
            .cloned()
            .ok_or("No such weapon")?;

        let mut dmg_dice = weapon.damage(&user_data.lang);
        if weapon.apply_damage_modifier {
            let damage_modifier = character.damage_modifier(&user_data.lang);
            if damage_modifier != "0" {
                dmg_dice.push_str(&damage_modifier);
                if weapon.half_damage_modifier {
                    dmg_dice.push_str("x0.5");
                }
            }
        };

        let mut add_desc: Option<String> = None;
        let dmg_result = if extreme_damage.unwrap_or_default() {
            add_desc = Some(format!(
                "\n({})",
                locale_text_lang(&user_data.lang, &crate::locale::LocaleTag::ExtremeDamage)?
            ));
            if weapon.impaling {
                let mut rng = RealRng::new();
                merge_dice_results(&[
                    get_roll_max(&dmg_dice)?,
                    roll_impl(&mut rng, &weapon.damage(&user_data.lang))?,
                ])?
            } else {
                get_roll_max(&dmg_dice)?
            }
        } else {
            let mut rng = RealRng::new();
            roll_impl(&mut rng, &dmg_dice)?
        };

        let message = Message {
            title: format!("**{}**", dmg_result.result,),
            description: format!(
                "**{} ({}){}**\n{}: {}",
                weapon.name.get(&user_data.lang),
                dmg_dice,
                add_desc.unwrap_or_default(),
                locale_text_lang(&user_data.lang, &crate::locale::LocaleTag::Rolls)?,
                fold(dmg_result.rolls, String::new(), |s, r| format!("{} `[{}]`", s, r)),
            ),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().save().await?;
    Ok(())
}
