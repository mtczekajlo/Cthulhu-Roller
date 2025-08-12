use crate::{
    commands::character::skill::skill_impl::skill_impl_tag,
    locale::{LocaleTag, locale_text_by_tag_lang},
    types::*,
};

#[poise::command(slash_command, rename = "spot_hidden", name_localized("pl", "spostrzegawczość"))]
pub async fn spot_hidden_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    skill_impl_tag(ctx, LocaleTag::SpotHidden, &modifier_dice.as_deref()).await
}

#[poise::command(slash_command, rename = "listen", name_localized("pl", "nasłuchiwanie"))]
pub async fn listen_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    skill_impl_tag(ctx, LocaleTag::Listen, &modifier_dice.as_deref()).await
}

#[poise::command(slash_command, rename = "dodge", name_localized("pl", "unik"))]
pub async fn dodge_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "dodatkowe_kości")] modifier_dice: Option<String>,
) -> Result<(), Error> {
    skill_impl_tag(ctx, LocaleTag::Dodge, &modifier_dice.as_deref()).await
}

#[poise::command(slash_command, rename = "maneuver", name_localized("pl", "manewr"))]
pub async fn maneuver_cmd(
    ctx: Context<'_>,
    #[name_localized("pl", "krzepa_celu")] target_build: Option<i32>,
    #[name_localized("pl", "dodatkowe_kości")] mut modifier_dice: Option<String>,
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

        if let Some(target_build) = target_build {
            let build_diff = target_build - character.build;
            let additional_dice = if build_diff > 2 {
                return Err(locale_text_by_tag_lang(user_data.lang, LocaleTag::ManeuverBuildError).into());
            } else if build_diff == 2 {
                "--"
            } else if build_diff == 1 {
                "-"
            } else {
                ""
            };

            if let Some(modifier_dice) = &mut modifier_dice {
                modifier_dice.push_str(additional_dice);
            } else {
                modifier_dice = Some(additional_dice.into());
            }
        };
    }

    skill_impl_tag(ctx, LocaleTag::FightingBrawl, &modifier_dice.as_deref()).await
}
