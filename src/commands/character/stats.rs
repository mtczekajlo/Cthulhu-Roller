use crate::{
    bot_data::*,
    character::Character,
    commands::basic::croll_impl,
    locale::*,
    message::MessageContent,
    roller::{dice_rng::RealRng, roll::roll_impl, success_level::SuccessLevel},
    types::*,
};
use poise::CreateReply;

pub fn hp_impl(character: &mut Character, delta: i32, lang: LocaleLang) -> Result<MessageContent, Error> {
    character.hp.modify(delta);

    let mut additional_desc = String::new();
    if delta < -(character.hp.max) {
        character.dead = true;
        character.major_wound = false;
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::DeathInevitable));
    } else {
        if delta <= -(character.hp.max / 2) {
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
        description: format!("❤️ **{:+}**\n{}{}", delta, character.status_hp(), additional_desc),
        ..Default::default()
    };

    Ok(mc)
}

#[poise::command(slash_command, rename = "hp", name_localized("pl", "pw"))]
pub async fn hp_cmd(ctx: Context<'_>, #[name_localized("pl", "zmiana")] delta: i32) -> Result<(), Error> {
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

        mc = hp_impl(character, delta, user_data.lang)?;
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

pub fn san_impl(character: &mut Character, delta: i32, lang: LocaleLang) -> Result<MessageContent, Error> {
    character.sanity.modify(delta);

    let mut additional_desc = String::new();
    if character.sanity.current == 0 {
        character.insane = true;
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::MindShattered));
    } else if character.fragile_mind && delta < 0 {
        additional_desc = format!("\n\n**{}**", locale_text_by_tag_lang(lang, LocaleTag::TempInsanity));
    } else if delta <= -5 {
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
        description: format!("🧠 **{:+}**\n{}{}", delta, character.status_sanity(), additional_desc),
        ..Default::default()
    };

    Ok(mc)
}

#[poise::command(slash_command, rename = "sanity", name_localized("pl", "poczytalność"))]
pub async fn sanity_cmd(ctx: Context<'_>, #[name_localized("pl", "zmiana")] delta: Option<i32>) -> Result<(), Error> {
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
            mc = san_impl(character, *delta, user_data.lang)?;
        } else {
            let sanity_str = locale_text_by_tag_lang(user_data.lang, LocaleTag::Sanity);
            let sanity_result = croll_impl(&character.sanity.current.to_string())?;
            mc = MessageContent::from_croll_result(user_data.lang, &sanity_result, false, true)
                .with_skill_name(&sanity_str)
                .with_character_name(&character_name);
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(slash_command, rename = "luck", name_localized("pl", "szczęście"))]
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
            let delta = delta.replace(' ', "").parse::<i32>()?;

            if delta < -character.luck.current {
                return Err(locale_text_by_tag_lang(user_data.lang, LocaleTag::CantSpendLuck).into());
            }

            character.luck.modify(delta);

            mc.title = format!("`{}`", character.name);
            mc.description = format!("🍀 **{:+}**\n{}", delta, character.status_luck());
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
                    res = roll_impl(&mut rng, &improve_dice)?;
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

#[poise::command(slash_command, rename = "mp", name_localized("pl", "pm"))]
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
