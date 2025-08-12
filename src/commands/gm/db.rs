use crate::{
    bot_data::*,
    commands::gm::is_user_gm,
    message::MessageContent,
    types::{Context, Error},
};
use poise::{
    CreateReply,
    serenity_prelude::{Attachment, CreateAttachment},
};

#[poise::command(
    slash_command,
    rename = "gmquicksave",
    name_localized("pl", "gmszybkizapis"),
    check = "is_user_gm"
)]
pub async fn gmquicksave_cmd(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().data.write().await.quicksave().await?;
    ctx.send(
        CreateReply::default()
            .embed(
                MessageContent {
                    description: "DB quicksave".to_string(),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    rename = "gmquickload",
    name_localized("pl", "gmszybkiodczyt"),
    check = "is_user_gm"
)]
pub async fn gmquickload_cmd(ctx: Context<'_>) -> Result<(), Error> {
    ctx.data().data.write().await.quickload().await?;
    ctx.send(
        CreateReply::default()
            .embed(
                MessageContent {
                    description: "DB quickload".to_string(),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    rename = "gmdatabase",
    name_localized("pl", "gmbazadanych"),
    check = "is_user_gm"
)]
pub async fn gmdatabase_cmd(
    ctx: poise::Context<'_, ContextData, Error>,
    #[name_localized("pl", "plik")] file: Option<Attachment>,
) -> Result<(), Error> {
    if let Some(file) = file {
        let response = reqwest::get(&file.url).await?;
        let content = response.text().await?;
        let db_parsed = serde_json::from_str::<Data>(&content)?;
        ctx.data().data.write().await.clone_from(&db_parsed);
        ctx.data().data.write().await.save().await?;

        ctx.send(
            CreateReply::default()
                .embed(
                    MessageContent {
                        description: "DB saved.".to_string(),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
    } else {
        let file_attachment = CreateAttachment::path(ctx.data().data.read().await.get_db_path()).await?;
        ctx.send(
            CreateReply::default()
                .content("📦 Here’s your current database backup:")
                .attachment(file_attachment)
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}
