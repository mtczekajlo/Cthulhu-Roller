#[cfg(feature = "character-sheet")]
pub mod character;
#[cfg(feature = "character-sheet")]
pub use character::*;
use itertools::Itertools;

use crate::{bot_data::UserData, locale::LOCALE_ATTRIBUTES, types::Context};

pub async fn autocomplete_help<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    ctx.framework()
        .options()
        .commands
        .iter()
        .filter(|cmd| cmd.name.to_ascii_lowercase().starts_with(&partial.to_ascii_lowercase()))
        .map(|cmd| cmd.name.clone())
        .collect()
}

pub async fn autocomplete_attributes<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let user_id = ctx.author().id.get();
    let mut data = ctx.data().data.write().await;
    let users = &mut data.users;
    if users.get_mut(&user_id).is_none() {
        users.insert(user_id, UserData::default());
    }
    let user_data = users.get_mut(&user_id).unwrap();

    let mut attributes: Vec<_> = LOCALE_ATTRIBUTES
        .iter()
        .map(|(_tag, text)| text.get(user_data.lang))
        .filter(|name| name.to_lowercase().contains(&partial.to_lowercase()))
        .collect();
    attributes.sort();
    attributes
}

pub async fn autocomplete_battle<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await;
    if let Some(battle) = &data.battle {
        battle
            .characters
            .iter()
            .map(|el| &el.name)
            .filter(|el| el.to_ascii_lowercase().contains(&partial.to_ascii_lowercase()))
            .sorted()
            .cloned()
            .collect()
    } else {
        vec![]
    }
}
