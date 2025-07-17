use crate::{bot_data::*, commands::is_user_gm, message::Message, types::*};
use poise::{
    CreateReply,
    serenity_prelude::{Attachment, CreateAttachment},
};

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn quicksave(ctx: Context<'_>) -> Result<(), Error> {
    {
        ctx.data().quicksave().await?;
    }
    ctx.send(
        CreateReply::default()
            .embed(
                Message {
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

#[poise::command(slash_command, check = "is_user_gm")]
pub async fn quickload(ctx: Context<'_>) -> Result<(), Error> {
    {
        ctx.data().quickload().await?;
    }
    ctx.send(
        CreateReply::default()
            .embed(
                Message {
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

#[poise::command(prefix_command, slash_command, check = "is_user_gm")]
pub async fn db(
    ctx: poise::Context<'_, Data, Error>,
    #[name_localized("pl", "plik")] file: Option<Attachment>,
) -> Result<(), Error> {
    if let Some(file) = file {
        let response = reqwest::get(&file.url).await?;
        let content = response.text().await?;
        let db_parsed = serde_json::from_str::<InnerData>(&content)?;

        {
            let mut inner_data;
            inner_data = ctx.data().data.write().await;
            inner_data.clone_from(&db_parsed);
        }

        ctx.data().save().await?;

        ctx.send(
            CreateReply::default()
                .embed(
                    Message {
                        description: "DB saved.".to_string(),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
    } else {
        let file_attachment = CreateAttachment::path(ctx.data().get_db_path()).await?;
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
