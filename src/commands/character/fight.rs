use std::sync::Arc;

use crate::{
    character::WeaponOk,
    commands::{
        autocomplete::*,
        basic::croll_impl,
        character::interaction::{add_spend_luck_buttons, handle_interaction, ok_button},
    },
    locale::{LocaleTag, locale_tag_by_str, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{
        dice_rng::RealRng,
        modifier_dice::ModifierDiceType,
        roll::{get_roll_max, merge_roll_results, roll_die, roll_query},
        success_level::SuccessLevel,
    },
    types::*,
};
use itertools::{Itertools, any, enumerate};
use poise::{CreateReply, serenity_prelude::CreateActionRow};

#[poise::command(prefix_command, slash_command, rename = "fight", aliases("walka"))]
pub async fn fight_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
    #[autocomplete = "autocomplete_my_weapon_attacks"]
    #[name_localized("pl", "ataki")]
    attacks: String,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    let user_id;
    let user_lang;
    let character_name;
    let mut mcs = vec![];
    let skill_already_marked;
    let weapon_skill_str;
    let task_results;

    {
        user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let data_users = &mut data.users;
        data_users.entry(user_id).or_default();
        let user_data = data_users.get_mut(&user_id).unwrap();
        user_lang = user_data.lang;
        character_name = user_data
            .active_character
            .clone()
            .ok_or(locale_text_by_tag_lang(user_data.lang, LocaleTag::NoCharacterSelected))?;
        let character = user_data.characters.get_mut(&character_name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &character_name
        ))?;

        let attack_num = attacks.parse::<i32>()?;
        let attack;

        let skill_improvable;
        {
            let weapon = character
                .weapons
                .iter()
                .find(|&w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
                .cloned()
                .ok_or("No such weapon")?;

            attack = weapon
                .attacks
                .iter()
                .find(|a| a.count == attack_num)
                .ok_or(Error::from("incorrect attacks number"))?
                .clone();
            weapon_skill_str = weapon.skill.clone();
        }

        {
            let skill = character.get_mut_skill(&weapon_skill_str).ok_or_else(|| {
                format!(
                    "{}: {}",
                    locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                    weapon_skill_str
                )
            })?;
            skill_improvable = skill.improvable;
            skill_already_marked = skill.to_improve;
        }

        for attack_number in 0..attack.count {
            let mut is_this_attack_last = false;

            let mut buttons = vec![];

            let modifier_dice = if let Some(modifier_dice) = modifier_dice.clone() {
                modifier_dice + attack.get_modifier(attack_number).as_str()
            } else {
                attack.get_modifier(attack_number)
            };

            let skill = character.get_mut_skill(&weapon_skill_str).ok_or_else(|| {
                format!(
                    "{}: {}",
                    locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                    weapon_skill_str
                )
            })?;
            let croll_query = format!("{}{}", skill.value, modifier_dice);
            let mut croll_result = croll_impl(&croll_query)?;

            let weapon_result;
            let weapon_malfunction;
            {
                let weapon = character
                    .weapons
                    .iter_mut()
                    .find(|w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
                    .ok_or("No such weapon")?;
                weapon_malfunction = weapon.malfunction;
                weapon_result = match weapon.ammo {
                    Some(_) => weapon.use_ammo(1),
                    None => Ok(WeaponOk::Hit),
                };
            }

            if weapon_result.is_err() {
                croll_result.success_level = SuccessLevel::Failure;
            } else if let Some(weapon_malfunction) = weapon_malfunction
                && croll_result.result() >= weapon_malfunction
            {
                is_this_attack_last = true;
                croll_result.success_level = SuccessLevel::CriticalFailure;
            } else if croll_result.success_level < SuccessLevel::ExtremeSuccess
                && croll_result.success_level != SuccessLevel::CriticalFailure
            {
                buttons.push(ok_button());
                buttons.extend(add_spend_luck_buttons(&croll_result, character, user_lang));
            }

            let mut mark_to_improve = false;
            if skill_improvable && croll_result.success_level >= SuccessLevel::Success {
                match &croll_result.modifier_dice {
                    None => mark_to_improve = true,
                    Some(modifier_dice) => {
                        if modifier_dice.dice_type == ModifierDiceType::Penalty {
                            mark_to_improve = true
                        }
                    }
                }
            }

            let mut mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                .with_skill_name(&weapon_skill_str)
                .with_character_name(&character_name);

            if let Err(e) = weapon_result {
                mc.description = format!("**{}**", e.to_string(user_lang));
            } else if let Some(malfunction) = weapon_malfunction
                && croll_result.result() >= malfunction
            {
                let jammed_rounds;
                {
                    let mut rng = RealRng::new();
                    jammed_rounds = roll_die(&mut rng, 6);
                }
                mc.description = format!(
                    "{}\n**{}**\n{}: **{}**",
                    mc.description,
                    locale_text_by_tag_lang(user_lang, LocaleTag::WeaponJammed),
                    locale_text_by_tag_lang(user_lang, LocaleTag::RoundsToUnjam),
                    jammed_rounds
                );
            } else {
                mc.description = format!(
                    "{}\n\n{}",
                    match croll_result.success_level {
                        SuccessLevel::Success | SuccessLevel::HardSuccess => WeaponOk::Hit.to_string(user_lang),
                        SuccessLevel::ExtremeSuccess | SuccessLevel::CriticalSuccess =>
                            format!("**{}**", WeaponOk::CriticalHit.to_string(user_lang)),
                        _ => WeaponOk::Miss.to_string(user_lang),
                    },
                    mc.description,
                );
            }

            mcs.push((croll_result, mc, buttons, mark_to_improve));

            if is_this_attack_last {
                break;
            }
        }
    }

    let mut replies = Arc::new(vec![]);
    for (_, mut mc, buttons, mark_to_improve) in mcs.clone() {
        let reply = if !buttons.is_empty() {
            let action_row = CreateActionRow::Buttons(buttons.clone());
            ctx.send(CreateReply::default().embed(mc.to_embed()).components(vec![action_row]))
                .await?
        } else {
            if !skill_already_marked && mark_to_improve {
                mc.description = format!(
                    "{}\n{}",
                    mc.description,
                    locale_text_by_tag_lang(user_lang, LocaleTag::SkillMarked)
                );
            }
            ctx.send(CreateReply::default().embed(mc.to_embed())).await?
        };
        Arc::make_mut(&mut replies).push(reply);
    }

    (_, task_results) = unsafe {
        async_scoped::TokioScope::scope_and_collect(|scope| {
            for (i, (croll_result, mc, buttons, mark_to_improve)) in enumerate(mcs) {
                let data = ctx.data().data.clone();
                let weapon_skill_str = weapon_skill_str.clone();
                let character_name = character_name.clone();
                let mut mc = mc;
                let mut mark_to_improve = mark_to_improve;
                let replies = replies.clone();

                scope.spawn(async move {
                    let mut interaction_result = None;
                    let reply = &replies[i];
                    if !buttons.is_empty() {
                        interaction_result = handle_interaction(ctx.serenity_context().shard.clone(), reply)
                            .await
                            .unwrap();
                        if interaction_result.is_none() {
                            MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                                .with_skill_name(&weapon_skill_str)
                                .with_character_name(&character_name);
                            if !skill_already_marked && mark_to_improve {
                                mc.description = format!(
                                    "{}\n{}",
                                    mc.description,
                                    locale_text_by_tag_lang(user_lang, LocaleTag::SkillMarked)
                                );
                            }
                            reply
                                .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                                .await
                                .unwrap();
                        }
                    }

                    if let Some(ir) = interaction_result
                        && let Some(tag) = locale_tag_by_str(&ir)
                    {
                        match tag {
                            LocaleTag::Success | LocaleTag::HardSuccess | LocaleTag::ExtremeSuccess => {
                                mark_to_improve = false;
                                let sl = SuccessLevel::from_tag(tag).unwrap();
                                let luck = sl.delta(croll_result.result(), croll_result.threshold);
                                let remaining_luck;
                                let mut croll_result = croll_result;
                                croll_result.set_result(croll_result.result() - luck);
                                croll_result.success_level = sl;
                                {
                                    let mut data = data.write().await;
                                    let user_data = data.users.entry(user_id).or_default();
                                    let character = user_data
                                        .characters
                                        .get_mut(&character_name)
                                        .ok_or(format!(
                                            "{}: `{}`",
                                            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
                                            &character_name
                                        ))
                                        .expect("EEE");
                                    character.luck.modify(-luck);
                                    remaining_luck = character.luck.current;
                                }
                                let mut mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                                    .with_skill_name(&weapon_skill_str)
                                    .with_character_name(&character_name);
                                mc.title = format!("{} (🍀)", mc.title);
                                mc.description = format!("{}\n\n🍀-{} ({})", mc.description, luck, remaining_luck);
                                reply
                                    .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                                    .await
                                    .unwrap();
                            }
                            _ => (),
                        }
                    }

                    mark_to_improve
                });
            }
        })
        .await
    };

    let mark_to_improve = any(task_results, |m| m.unwrap());

    if !skill_already_marked && mark_to_improve {
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let character = user_data.characters.get_mut(&character_name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &character_name
        ))?;
        let skill = character.get_mut_skill(&weapon_skill_str).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                weapon_skill_str
            )
        })?;
        skill.to_improve |= mark_to_improve;
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "damage", aliases("obrażenia"))]
pub async fn damage_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_weapons"]
    #[name_localized("pl", "nazwa_broni")]
    weapon_name: String,
    #[description = "'n' for normal, 'e' for extreme; e.g. 'nne'"]
    #[description_localized("pl", "'n' dla normalnych, 'e' dla ekstremalnych; np. 'nne'")]
    #[name_localized("pl", "typy_obrażeń")]
    damage_types: Option<String>,
    #[name_localized("pl", "dystans")]
    #[description = "distance in meters (~yards)"]
    #[description_localized("pl", "dystans w metrach")]
    distance: Option<i32>,
) -> Result<(), Error> {
    let damage_types = damage_types.unwrap_or('n'.to_string());

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
            .iter()
            .find(|&w| w.name.to_ascii_lowercase().contains(&weapon_name.to_ascii_lowercase()))
            .cloned()
            .ok_or("No such weapon")?;

        let weapon_damage = weapon.damage_dice(user_data.lang, distance);

        let mut dmg_dice = weapon_damage.clone();
        if weapon.apply_damage_modifier {
            let damage_modifier = character.damage_modifier(user_data.lang);
            if damage_modifier != "0" {
                dmg_dice.push_str(&damage_modifier);
                if weapon.half_damage_modifier {
                    dmg_dice.push_str("x0.5");
                }
            }
        };

        let mut dmg_results = vec![];
        for c in damage_types.chars() {
            let rr = match c {
                'n' => {
                    let mut rng = RealRng::new();
                    Some(roll_query(&mut rng, &dmg_dice)?)
                }
                'e' => Some(if weapon.impaling {
                    let mut rng = RealRng::new();
                    merge_roll_results(&[get_roll_max(&dmg_dice)?, roll_query(&mut rng, &weapon_damage)?])?
                } else {
                    get_roll_max(&dmg_dice)?
                }),
                _ => None,
            };
            if let Some(rr) = rr {
                dmg_results.push(rr);
            }
        }
        let dmg_text = dmg_results
            .iter()
            .map(|d| {
                format!(
                    "{} {}",
                    locale_text_by_tag_lang(user_data.lang, LocaleTag::Rolls),
                    d.roll_msg
                )
            })
            .join("\n");
        let dmg_result = merge_roll_results(&dmg_results)?;

        let message = MessageContent {
            title: format!("**{}**", dmg_result.result(),),
            description: format!("**{} ({})**\n\n{}", weapon.name, dmg_dice, dmg_text),
            ..Default::default()
        };

        ctx.send(CreateReply::default().embed(message.to_embed())).await?;
    }

    ctx.data().data.write().await.save().await
}
