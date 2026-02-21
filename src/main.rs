mod bot_data;
#[cfg(feature = "character-sheet")]
mod character;
mod command_list;
mod commands;
mod locale;
mod message;
mod roller;
mod types;
mod utils;

use crate::command_list::{CommandCategory, CommandMeta};
#[cfg(feature = "character-sheet")]
use crate::commands::gm::is_user_gm;
use bot_data::{ContextData, Data};
use command_list::command_list;
use commands::autocomplete::autocomplete_help;
use poise::serenity_prelude::{Http, HttpBuilder};
use poise::{
    CreateReply,
    serenity_prelude::{ClientBuilder, GatewayIntents},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use types::{Context, Error, FrameworkError};

#[poise::command(prefix_command, slash_command, aliases("pomoc"))]
pub async fn help(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_help"] command: Option<String>,
) -> Result<(), Error> {
    let all_commands = &ctx.framework().options().commands;

    if let Some(command) = command {
        if let Some(cmd) = all_commands
            .iter()
            .find(|el| el.name.to_ascii_lowercase().eq(command.to_ascii_lowercase().as_str()))
        {
            let mut short_desc: Option<&'static str> = None;
            let mut long_desc: Option<&'static str> = None;
            if let Some(meta) = cmd.custom_data.downcast_ref::<CommandMeta>() {
                short_desc = Some(meta.short_desc);
                long_desc = Some(meta.long_desc);
            }
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "`/{}`\n\n{}\n\n{}",
                        cmd.name,
                        short_desc.unwrap_or_default(),
                        long_desc.unwrap_or_default()
                    ))
                    .ephemeral(true),
            )
            .await?;
        } else {
            return Err(format!("No such command: `{command}`").into());
        }
    } else {
        let commands = vec![
            format_help(all_commands, CommandCategory::Basic),
            #[cfg(feature = "character-sheet")]
            format_help(all_commands, CommandCategory::Character),
        ];

        #[cfg(feature = "character-sheet")]
        let commands = if is_user_gm(ctx).await? {
            commands
                .into_iter()
                .chain(vec![format_help(all_commands, CommandCategory::GM)])
                .collect()
        } else {
            commands
        };

        for command in commands {
            ctx.send(CreateReply::default().content(command).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}

fn format_help(commands: &[poise::Command<ContextData, Error>], category: CommandCategory) -> String {
    let mut out = format!("**{category:?} Commands**\n");

    let mut table = tabled::builder::Builder::new();

    for cmd in commands.iter().filter(|cmd| {
        if let Some(meta) = &cmd.custom_data.downcast_ref::<CommandMeta>() {
            meta.category == category
        } else {
            false
        }
    }) {
        table.push_record([
            ["/".to_owned(), cmd.name.clone()].join(""),
            cmd.custom_data
                .downcast_ref::<CommandMeta>()
                .unwrap()
                .short_desc
                .to_string(),
        ]);
    }

    out.push_str("```text\n");
    out.push_str(
        table
            .build()
            .with(tabled::settings::Style::empty())
            .to_string()
            .as_str(),
    );
    out.push_str("```");

    out
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;

    let data = Data::load().await;

    let commands_vec = command_list();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands_vec,
            on_error: |error| Box::pin(handle_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(ContextData {
                    data: Arc::new(RwLock::new(data)),
                })
            })
        })
        .build();

    let http = if let Ok(proxy) = std::env::var("HTTP_PROXY") {
        println!("Using proxy: {proxy}");
        HttpBuilder::new(&discord_token).proxy(&proxy).build()
    } else {
        Http::new(&discord_token)
    };

    let mut client = ClientBuilder::new_with_http(http, GatewayIntents::non_privileged())
        .framework(framework)
        .await?;

    Ok(client.start().await?)
}

fn any_to_str<T>(_: &T) -> String {
    std::any::type_name::<T>().into()
}

async fn handle_error(error: FrameworkError<'_>) {
    match error {
        poise::FrameworkError::Command { ctx, error, .. } => {
            eprintln!("{error}");
            let help_message = ctx.command().help_text.clone().unwrap_or_default();
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content(format!("{error}\n\n{help_message}"))
                        .ephemeral(true),
                )
                .await;
        }
        poise::FrameworkError::CommandCheckFailed { ctx, .. } => {
            eprintln!("{error}");
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("🚫 This command is only available to GMs.")
                        .ephemeral(true),
                )
                .await;
        }
        error => {
            eprintln!("{}: {error}", any_to_str(&error));
            std::process::exit(1)
        }
    }
}
