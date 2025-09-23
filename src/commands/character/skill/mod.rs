pub mod add;
pub mod skill_impl;
use crate::{
    character::Skill,
    commands::{autocomplete::character::*, character::skill::skill_impl::skill_impl_str},
    locale::{LocaleLang, LocaleTag, locale_tag_by_str, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{dice_rng::RealRng, improve_roll::improve_skill, roll::roll_query, success_level::SuccessLevel},
    types::{Context, Error},
};
use add::add_cmd;
use poise::CreateReply;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "skill",
    name_localized("pl", "umiejętność"),
    subcommands(
        "list_cmd",
        "check_cmd",
        "add_cmd",
        "set_cmd",
        "change_cmd",
        "remove_cmd",
        "improve_cmd",
        "mark_cmd",
        "unmark_cmd"
    )
)]
pub async fn skill_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "list", name_localized("pl", "lista"))]
async fn list_cmd(ctx: Context<'_>) -> Result<(), Error> {
    let mcs;
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

        mcs = MessageContent::from_character_skills(user_data.lang, &character.skills);
    }

    for mc in mcs {
        ctx.send(CreateReply::default().content(mc.to_content()).ephemeral(true))
            .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "check", name_localized("pl", "test"))]
async fn check_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills_with_additional"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    skill_impl_str(ctx, &name, &modifier_dice.as_deref()).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command, rename = "set", name_localized("pl", "ustaw"))]
async fn set_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] value: i32,
) -> Result<(), Error> {
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

        character
            .set_skill(&name, value)
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("`{name}` ⬅️ `{value}`"),
            ..Default::default()
        };

        if locale_tag_by_str(&name) == Some(LocaleTag::CthulhuMythos) {
            mc.description = format!(
                "{}\n\n{} **{}**",
                mc.description,
                locale_text_by_tag_lang(user_data.lang, LocaleTag::MaxSanitySet),
                character.sanity.max
            );
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "change", name_localized("pl", "zmiana"))]
async fn change_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] mut query: String,
) -> Result<(), Error> {
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

        if user_data.lang == LocaleLang::Polski {
            query = query.replace('d', "k");
        }

        let roll_result;
        {
            let mut rng = RealRng::new();
            roll_result = roll_query(&mut rng, &query)?;
        }

        character
            .modify_skill(&name, roll_result.result_real())
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        {
            let skill = character.get_skill(&name).ok_or_else(|| {
                format!(
                    "{}: {}",
                    locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                    name
                )
            })?;

            mc = MessageContent {
                title: format!("`{}`", character_name),
                description: format!(
                    "**{name}**\n{} **{:+}** = **{}** ({})",
                    skill.value - roll_result.result_real(),
                    roll_result.result_real(),
                    skill.value,
                    query,
                ),
                ..Default::default()
            };
        }

        if locale_tag_by_str(&name) == Some(LocaleTag::CthulhuMythos) {
            mc.description = format!(
                "{}\n\n{} **{}**",
                mc.description,
                locale_text_by_tag_lang(user_data.lang, LocaleTag::MaxSanitySet),
                character.sanity.max
            );
        }
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "remove", name_localized("pl", "usuń"))]
async fn remove_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_custom_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
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

        character
            .remove_skill(&name)
            .map_err(|e| Error::from(e.to_string(user_data.lang)))?;

        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("❌ `{name}`"),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "improve", name_localized("pl", "rozwiń"))]
async fn improve_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "kość_rozwoju")] improve_dice: Option<String>,
) -> Result<(), Error> {
    let mut mcs = vec![];
    let mut ephemeral = false;

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

        let skills: Vec<(&String, &mut Skill)> = character
            .skills
            .iter_mut()
            .filter(|s| s.1.improvable && s.1.to_improve)
            .collect();

        if skills.is_empty() {
            let mc = MessageContent {
                title: locale_text_by_tag_lang(user_data.lang, LocaleTag::NotMarked),
                ..Default::default()
            };
            ephemeral = true;
            mcs.push(mc);
        } else {
            for (name, skill) in skills {
                if skill.improvable && skill.to_improve {
                    let improve_result = improve_skill(skill.value);
                    let mut mc = MessageContent::from_improve(user_data.lang, &improve_result);
                    skill.to_improve = false;

                    let mut improve_dice = improve_dice.clone().unwrap_or("d6".into()); //TODO Home Rules; rule-wise: d10
                    if user_data.lang == LocaleLang::Polski {
                        improve_dice = improve_dice.replace('d', "k");
                    }
                    if improve_result.success_level == SuccessLevel::Success {
                        let res;
                        {
                            let mut rng = RealRng::new();
                            res = roll_query(&mut rng, &improve_dice)?;
                        }
                        let skill_value = skill.value;
                        let skill_new_value = skill_value + res.result();
                        skill.set(skill_new_value).map_err(|e| e.to_string(user_data.lang))?;

                        mc.title = format!("{}\n{}", mc.title, name);
                        mc.description = format!(
                            "{}\n{} **{:+}** = **{}** ({})",
                            mc.description,
                            skill_value,
                            res.result(),
                            skill_new_value,
                            improve_dice,
                        );
                        mcs.push(mc);
                    }
                }
            }
        }
    }

    for mc in mcs {
        ctx.send(CreateReply::default().embed(mc.to_embed()).ephemeral(ephemeral))
            .await?;
    }

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "mark", name_localized("pl", "zaznacz"))]
async fn mark_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
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

        let skill = character.get_mut_skill(&name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                name
            )
        })?;

        skill.to_improve = true;

        mc = MessageContent {
            title: format!("**{name}**",),
            description: locale_text_by_tag_lang(user_data.lang, LocaleTag::SkillMarked),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}

#[poise::command(prefix_command, slash_command, rename = "unmark", name_localized("pl", "odznacz"))]
async fn unmark_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_improvable_skills"]
    #[name_localized("pl", "nazwa")]
    name: String,
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

        let skill = character.get_mut_skill(&name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                name
            )
        })?;

        skill.to_improve = false;

        mc = MessageContent {
            title: format!("**{name}**",),
            description: locale_text_by_tag_lang(user_data.lang, LocaleTag::SkillUnmarked),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await
}
