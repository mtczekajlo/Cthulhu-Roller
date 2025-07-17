use crate::roller::SuccessLevel;
use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};

pub mod formatter;
pub use formatter::*;
pub mod help;
pub use help::*;

#[derive(Default)]
pub struct Message {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub colour: Option<u32>,
}

impl Message {
    pub fn to_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        embed = embed.title(&self.title);
        embed = embed.description(&self.description);
        embed = embed.footer(CreateEmbedFooter::new(&self.footer));
        embed = embed.colour(match self.colour {
            Some(colour) => Colour::from(colour),
            None => Colour::from(SuccessLevel::Success.hex()),
        });
        embed
    }
}
