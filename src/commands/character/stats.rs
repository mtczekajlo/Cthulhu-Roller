use crate::{
    bot_data::*,
    character::Character,
    commands::basic::croll_impl,
    locale::*,
    message::MessageContent,
    roller::{dice_rng::RealRng, improve_roll::improve_skill, roll::roll_query, success_level::SuccessLevel},
    types::*,
};
use poise::CreateReply;

pub fn hp_impl(character: &mut Character, delta: &str, lang: LocaleLang) -> Result<MessageContent, Error> {
    let roll_result;
    {
        let mut rng = RealRng::new();
        roll_result = roll_query(&mut rng, delta)?;
    }
    let delta_res = roll_result.result_real();

    character.hp.modify(delta_res);

    let mut additional_desc = String::new();
    if delta_res < -(character.hp.max) {
        character.dead = true;
        character.major_wound = false;
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::DeathInevitable));
    } else {
        if delta_res <= -(character.hp.max / 2) {
            character.major_wound = true;
            additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::MajorWound));
            if character.hp.current > 0 {
                additional_desc = format!(
                    "{}\n{}\n{}",
                    additional_desc,
                    locale_text_by_tag_lang(lang, LocaleTag::YouFell),
                    locale_text_by_tag_lang(lang, LocaleTag::RollConCheckBlackOut)
                )
                .to_string();
            }
        }

        if character.hp.current == 0 {
            if character.major_wound {
                additional_desc = format!(
                    "{}\n**{}**\n{}\n{}\n{}",
                    additional_desc,
                    locale_text_by_tag_lang(lang, LocaleTag::Agony),
                    locale_text_by_tag_lang(lang, LocaleTag::YouFell),
                    locale_text_by_tag_lang(lang, LocaleTag::YouBlackOut),
                    locale_text_by_tag_lang(lang, LocaleTag::RollConCheckDie)
                );
            } else {
                additional_desc = format!(
                    "{}\n**{}**\n{}\n{}",
                    additional_desc,
                    locale_text_by_tag_lang(lang, LocaleTag::KnockOut),
                    locale_text_by_tag_lang(lang, LocaleTag::YouFell),
                    locale_text_by_tag_lang(lang, LocaleTag::YouBlackOut),
                );
            }
        }
    }

    let mc = MessageContent {
        title: format!("`{}`", character.name),
        description: format!(
            "❤️ **{:+}** ({:+})\n{}{}",
            delta_res,
            delta,
            character.status_hp(),
            additional_desc
        ),
        ..Default::default()
    };

    Ok(mc)
}

#[poise::command(prefix_command, slash_command, rename = "hp", aliases("pw"))]
pub async fn hp_cmd(ctx: Context<'_>, #[name_localized("pl", "zmiana")] delta: String) -> Result<(), Error> {
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

        mc = hp_impl(character, &delta, user_data.lang)?;
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

pub fn san_impl(character: &mut Character, delta: &str, lang: LocaleLang) -> Result<MessageContent, Error> {
    let roll_result;
    {
        let mut rng = RealRng::new();
        roll_result = roll_query(&mut rng, delta)?;
    }
    let delta_res = roll_result.result_real();

    character.sanity.modify(delta_res);

    let mut additional_desc = String::new();
    if character.sanity.current == 0 {
        character.insane = true;
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::MindShattered));
    } else if character.fragile_mind && delta_res < 0 {
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::TempInsanity));
    } else if delta_res <= -5 {
        additional_desc = format!(
            "\n\n**{}**\n{}",
            locale_text_by_tag_lang(lang, LocaleTag::TempInsanityThreat),
            locale_text_by_tag_lang(lang, LocaleTag::RollIntCheck)
        );
    }

    if (character.sanity.current as f32 / character.sanity.initial as f32) < 0.8 {
        character.fragile_mind = true;
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::IndefInsanity));
    }

    let mc = MessageContent {
        title: format!("`{}`", character.name),
        description: format!(
            "🧠 **{:+}** ({:+})\n{}{}",
            delta_res,
            delta,
            character.status_sanity(),
            additional_desc
        ),
        ..Default::default()
    };

    Ok(mc)
}

#[poise::command(prefix_command, slash_command, rename = "sanity", aliases("poczytalność"))]
pub async fn sanity_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "zmiana")] delta: Option<String>,
) -> Result<(), Error> {
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

        if let Some(delta) = &delta {
            mc = san_impl(character, delta, user_data.lang)?;
        } else {
            let sanity_str = locale_text_by_tag_lang(user_data.lang, LocaleTag::Sanity);
            let sanity_result = croll_impl(&character.sanity.current.to_string())?;
            mc = MessageContent::from_croll_result(user_data.lang, &sanity_result, false, true)
                .with_skill_name(&sanity_str)
                .with_character_name(&character_name);
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "luck", aliases("szczęście"))]
pub async fn luck_cmd(
    ctx: poise::Context<'_, ContextData, Error>,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
    #[name_localized("pl", "zmiana")] delta: Option<String>,
) -> Result<(), Error> {
    let mut mc = MessageContent::default();

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

        if let Some(delta) = &delta {
            let roll_result;
            {
                let mut rng = RealRng::new();
                roll_result = roll_query(&mut rng, delta)?;
            }
            let delta_res = roll_result.result_real();

            if delta_res < -character.luck.current {
                return Err(locale_text_by_tag_lang(user_data.lang, LocaleTag::CantSpendLuck).into());
            }

            character.luck.modify(delta_res);

            mc.title = format!("`{}`", character.name);
            mc.description = format!("🍀 **{:+}** ({:+})\n{}", delta_res, delta, character.status_luck());
        } else {
            let luck_str = locale_text_by_tag_lang(user_data.lang, LocaleTag::Luck);
            let luck_query = format!(
                "{}{}",
                &character.luck.current.to_string(),
                modifier_dice.unwrap_or_default()
            );
            let luck_result = croll_impl(&luck_query)?;

            mc = MessageContent::from_croll_result(user_data.lang, &luck_result, false, true)
                .with_skill_name(&luck_str)
                .with_character_name(&character_name);

            if luck_result.success_level == SuccessLevel::CriticalSuccess {
                let mut improve_dice = "d10".to_string();
                if user_data.lang == LocaleLang::Polski {
                    improve_dice = improve_dice.replace('d', "k");
                }

                let res;
                {
                    let mut rng = RealRng::new();
                    res = roll_query(&mut rng, &improve_dice)?;
                }
                mc.description = format!(
                    "{}\n\n**{}**",
                    mc.description,
                    locale_text_by_tag_lang(user_data.lang, LocaleTag::LuckCritical)
                );
                let power = character.attributes.power();
                let new_power = power + res.result();
                mc.description = format!(
                    "{}\n{} + **{}** ({}) = **{}**",
                    mc.description,
                    character.attributes.power(),
                    res.result(),
                    improve_dice,
                    new_power
                );
                character.set_attribute("power", new_power);
            }
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "improve_luck", aliases("rozwiń_szczęście"))]
pub async fn improve_luck_cmd(ctx: poise::Context<'_, ContextData, Error>) -> Result<(), Error> {
    let mut mc;

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

        let improve_result = improve_skill(character.luck.current);
        mc = MessageContent::from_improve(user_data.lang, &improve_result);

        let improve_query = match (
            character.pulp_archetype.is_some(),
            (improve_result.success_level >= SuccessLevel::Success),
        ) {
            (true, true) => Some(String::from("2d10+10")),
            (true, false) => Some(String::from("1d10+5")),
            (false, true) => Some(String::from("1d10")),
            _ => None,
        };

        if let Some(mut iq) = improve_query {
            let luck_delta;
            {
                let mut rng = RealRng::new();
                luck_delta = roll_query(&mut rng, &iq)?;
                character.luck.modify(luck_delta.result());
            }

            if user_data.lang == LocaleLang::Polski {
                iq = iq.replace('d', "k");
            }

            mc.description = format!(
                "{}\n🍀 **{:+}** ({})\n{}",
                mc.description,
                luck_delta.result(),
                iq,
                character.status_luck()
            )
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "mp", aliases("pm"))]
pub async fn mp_cmd(
    ctx: poise::Context<'_, ContextData, Error>,
    #[name_localized("pl", "zmiana")] delta: i32,
) -> Result<(), Error> {
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

        character.magic.modify(delta);

        mc = MessageContent {
            title: format!("`{}`", &character_name),
            description: format!("🪄 **{:+}**\n{}", delta, character.status_magic()),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}
