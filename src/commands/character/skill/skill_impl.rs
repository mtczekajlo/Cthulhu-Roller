use poise::{CreateReply, serenity_prelude::CreateActionRow};

use crate::{
    commands::{
        basic::croll_impl,
        character::interaction::{add_push_roll_button, add_spend_luck_buttons, handle_interaction, ok_button},
    },
    locale::{LocaleLang, LocaleTag, locale_tag_by_str, locale_text_by_tag_lang},
    message::MessageContent,
    roller::{modifier_dice::ModifierDiceType, success_level::SuccessLevel},
    types::{Context, Error},
};

pub async fn skill_impl_str(ctx: Context<'_>, skill_name: &str, modifier_dice: &Option<&str>) -> Result<(), Error> {
    let user_id;
    let croll_query;
    let mut mc;
    let user_lang;
    let mut croll_result;
    let skill_improvable;
    let skill_already_marked;
    let character_name;
    let mut buttons = vec![];
    {
        user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
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
        let skill = character.get_mut_skill(skill_name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                skill_name
            )
        })?;
        skill_improvable = skill.improvable;
        skill_already_marked = skill.to_improve;
        croll_query = format!("{}{}", skill.value, modifier_dice.unwrap_or_default());
        croll_result = croll_impl(&croll_query)?;
        if croll_result.success_level < SuccessLevel::ExtremeSuccess
            && croll_result.success_level != SuccessLevel::CriticalFailure
        {
            buttons.push(ok_button());
            buttons.extend(add_push_roll_button(&croll_result, user_lang));
            buttons.extend(add_spend_luck_buttons(&croll_result, character, user_lang));
        }
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

    mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
        .with_skill_name(skill_name)
        .with_character_name(&character_name);

    let mut interaction_result = None;
    let reply;
    if !buttons.is_empty() {
        let action_row = CreateActionRow::Buttons(buttons);
        reply = ctx
            .send(CreateReply::default().embed(mc.to_embed()).components(vec![action_row]))
            .await?;
        interaction_result = handle_interaction(ctx.serenity_context().shard.clone(), &reply).await?;
        if interaction_result.is_none() {
            mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                .with_skill_name(skill_name)
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
                .await?;
        }
    } else {
        if !skill_already_marked && mark_to_improve {
            mc.description = format!(
                "{}\n{}",
                mc.description,
                locale_text_by_tag_lang(user_lang, LocaleTag::SkillMarked)
            );
        }
        reply = ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    }

    if let Some(ir) = interaction_result
        && let Some(tag) = locale_tag_by_str(&ir)
    {
        match tag {
            LocaleTag::PushRoll => {
                croll_result = croll_impl(&croll_query)?;
                mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                    .with_skill_name(skill_name)
                    .with_character_name(&character_name);
                mark_to_improve = false;
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
                if croll_result.success_level < SuccessLevel::Success {
                    mc.description = format!(
                        "{}\n\n{}: ***{}***",
                        mc.description,
                        locale_text_by_tag_lang(user_lang, LocaleTag::PushRoll),
                        locale_text_by_tag_lang(user_lang, LocaleTag::PrepareForTheConsequences)
                    );
                } else {
                    mc.description = format!(
                        "{}\n\n{}: *{}*",
                        mc.description,
                        locale_text_by_tag_lang(user_lang, LocaleTag::PushRoll),
                        locale_text_by_tag_lang(user_lang, LocaleTag::YouGotLuckyThisTIme)
                    );
                }
                if !skill_already_marked && mark_to_improve {
                    mc.description = format!(
                        "{}\n{}",
                        mc.description,
                        locale_text_by_tag_lang(user_lang, LocaleTag::SkillMarked)
                    );
                }
                reply
                    .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                    .await?;
            }
            LocaleTag::Success | LocaleTag::HardSuccess | LocaleTag::ExtremeSuccess => {
                mark_to_improve = false;
                let sl = SuccessLevel::from_tag(tag)?;
                let luck = sl.delta(croll_result.result(), croll_result.threshold);
                let remaining_luck;
                let mut croll_result = croll_result;
                croll_result.set_result(croll_result.result() - luck);
                croll_result.success_level = sl;
                {
                    let mut data = ctx.data().data.write().await;
                    let user_data = data.users.entry(user_id).or_default();
                    let character = user_data.characters.get_mut(&character_name).ok_or(format!(
                        "{}: `{}`",
                        locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
                        &character_name
                    ))?;
                    character.luck.modify(-luck);
                    remaining_luck = character.luck.current;
                }
                let mut mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                    .with_skill_name(skill_name)
                    .with_character_name(&character_name);
                mc.title = format!("{} (🍀)", mc.title);
                mc.description = format!("{}\n\n🍀-{} ({})", mc.description, luck, remaining_luck);
                reply
                    .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                    .await?;
            }
            _ => (),
        }
    }

    if !skill_already_marked && mark_to_improve {
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.entry(user_id).or_default();
        let character = user_data.characters.get_mut(&character_name).ok_or(format!(
            "{}: `{}`",
            locale_text_by_tag_lang(user_data.lang, LocaleTag::CharacterNotFound),
            &character_name
        ))?;

        let skill = character.get_mut_skill(skill_name).ok_or_else(|| {
            format!(
                "{}: {}",
                locale_text_by_tag_lang(user_data.lang, LocaleTag::NoSuchSkill),
                skill_name
            )
        })?;
        skill.to_improve |= mark_to_improve;
    }

    ctx.data().data.write().await.save().await
}

pub async fn skill_impl_tag(ctx: Context<'_>, skill_tag: LocaleTag, modifier_dice: &Option<&str>) -> Result<(), Error> {
    let user_locale = ctx.locale().unwrap();
    skill_impl_str(
        ctx,
        &locale_text_by_tag_lang(LocaleLang::from(user_locale), skill_tag),
        modifier_dice,
    )
    .await
}
