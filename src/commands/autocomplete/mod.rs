#[cfg(feature = "character-sheet")]
pub mod character;
#[cfg(feature = "character-sheet")]
pub use character::*;

use crate::types::Context;

pub async fn autocomplete_help<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    ctx.framework()
        .options()
        .commands
        .iter()
        .filter(|cmd| cmd.name.to_ascii_lowercase().starts_with(&partial.to_ascii_lowercase()))
        .map(|cmd| cmd.name.clone())
        .collect()
}
