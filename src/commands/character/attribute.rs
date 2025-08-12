use crate::{
    commands::{
        autocomplete::character::autocomplete_my_attributes,
        basic::croll_impl,
        character::interaction::{add_push_roll_button, add_spend_luck_buttons, handle_interaction, ok_button},
    },
    locale::{LocaleTag, locale_tag_by_str, locale_text_by_tag_lang},
    message::MessageContent,
    roller::success_level::SuccessLevel,
    types::*,
};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

#[poise::command(
    prefix_command,
    slash_command,
    rename = "attribute",
    aliases("characteristic"),
    name_localized("pl", "cecha"),
    subcommands("check_cmd", "set_cmd")
)]
pub async fn attribute_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, rename = "set", name_localized("pl", "ustaw"))]
async fn set_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_attributes"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "wartość")] value: i32,
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

        character.set_attribute(&name, value);
        mc = MessageContent {
            title: format!("`{}`", character.name),
            description: format!("`{name}` ⬅️ `{value}`"),
            ..Default::default()
        };
    }

    ctx.send(CreateReply::default().embed(mc.to_embed())).await?;

    ctx.data().data.write().await.save().await?;

    Ok(())
}

#[poise::command(slash_command, rename = "check", name_localized("pl", "test"))]
async fn check_cmd(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_my_attributes"]
    #[name_localized("pl", "nazwa")]
    name: String,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    let user_id;
    let croll_query;
    let mc;
    let user_lang;
    let croll_result;
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

        croll_query = format!(
            "{}{}",
            character.attributes.get(&name).unwrap().value,
            modifier_dice.unwrap_or_default()
        );
        croll_result = croll_impl(&croll_query)?;

        mc = MessageContent::from_croll_result(user_lang, &croll_result, false, false)
            .with_skill_name(&name)
            .with_character_name(&character_name);

        if croll_result.success_level < SuccessLevel::ExtremeSuccess {
            buttons.push(ok_button());
            buttons.extend(add_push_roll_button(&croll_result, user_lang));
            buttons.extend(add_spend_luck_buttons(&croll_result, character, user_lang));
        }
    }

    let mut interaction_result = None;
    let reply;
    if !buttons.is_empty() {
        let action_row = serenity::CreateActionRow::Buttons(buttons);
        reply = ctx
            .send(CreateReply::default().embed(mc.to_embed()).components(vec![action_row]))
            .await?;
        interaction_result = handle_interaction(ctx.serenity_context().shard.clone(), &reply).await?;
        if interaction_result.is_none() {
            reply
                .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                .await?;
        }
    } else {
        reply = ctx.send(CreateReply::default().embed(mc.to_embed())).await?;
    }

    if let Some(ir) = interaction_result
        && let Some(tag) = locale_tag_by_str(&ir)
    {
        match tag {
            LocaleTag::PushRoll => {
                let croll_result = croll_impl(&croll_query)?;
                let mut mc = MessageContent::from_croll_result(user_lang, &croll_result, false, true)
                    .with_skill_name(&name)
                    .with_character_name(&character_name);
                if croll_result.success_level < SuccessLevel::Success {
                    mc.description = format!(
                        "{}\n{}: ***{}***",
                        mc.description,
                        locale_text_by_tag_lang(user_lang, LocaleTag::PushRoll),
                        locale_text_by_tag_lang(user_lang, LocaleTag::PrepareForTheConsequences)
                    );
                } else {
                    mc.description = format!(
                        "{}\n{}: *{}*",
                        mc.description,
                        locale_text_by_tag_lang(user_lang, LocaleTag::PushRoll),
                        locale_text_by_tag_lang(user_lang, LocaleTag::YouGotLuckyThisTIme)
                    );
                }
                reply
                    .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                    .await?;
            }
            LocaleTag::Success | LocaleTag::HardSuccess | LocaleTag::ExtremeSuccess => {
                let sl = SuccessLevel::from_tag(tag)?;
                let luck = sl.delta(croll_result.result(), croll_result.threshold);
                let remaining_luck;
                let mut croll_result = croll_result;
                croll_result.set_result(croll_result.result() - luck);
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
                    .with_skill_name(&name)
                    .with_character_name(&character_name);
                mc.title = format!("{} (🍀)", mc.title);
                mc.description = format!("{}\n🍀-{} ({})", mc.description, luck, remaining_luck);
                reply
                    .edit(ctx, CreateReply::default().embed(mc.to_embed()).components(vec![]))
                    .await?;
            }
            _ => (),
        }
    }

    ctx.data().data.write().await.save().await
}
