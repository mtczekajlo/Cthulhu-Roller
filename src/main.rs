use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents},
    CreateReply,
};
mod roll;

#[cfg(feature = "character-sheet")]
mod character;
mod message;

mod locale;

mod autocomplete;
use autocomplete::*;

mod types;
use types::*;

mod help_messages;
use help_messages::*;

mod commands;
use commands::basic::*;

#[cfg(feature = "character-sheet")]
use commands::character::character_cmd::*;
#[cfg(feature = "character-sheet")]
use commands::character::skill::*;
#[cfg(feature = "character-sheet")]
use commands::character::*;
#[cfg(feature = "character-sheet")]
use commands::gm::gmcharacter::*;
#[cfg(feature = "character-sheet")]
use commands::gm::*;

mod bot_data;
use bot_data::*;

#[cfg(feature = "character-sheet")]
use crate::commands::character::{
    item_mod::item,
    weapon::{damage, weapon},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum CommandCategory {
    #[default]
    Basic,
    Character,
    GM,
}

#[derive(Clone, Default)]
pub struct CommandMeta {
    pub category: CommandCategory,
    pub short_desc: &'static str,
    pub long_desc: &'static str,
}

#[poise::command(slash_command, track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_help"] command: Option<String>,
) -> Result<(), Error> {
    let all_commands = &ctx.framework().options().commands;

    if let Some(command) = command {
        if let Some(cmd) = all_commands.iter().find(|el| {
            el.name
                .to_ascii_lowercase()
                .contains(command.to_ascii_lowercase().as_str())
        }) {
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

fn format_help(commands: &[poise::Command<Data, Error>], category: CommandCategory) -> String {
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

fn cmd_with_meta(
    mut cmd: poise::Command<Data, Error>,
    category: CommandCategory,
    short_desc: &'static str,
    long_desc: &'static str,
) -> poise::Command<Data, Error> {
    cmd.custom_data = Box::new(CommandMeta {
        category,
        short_desc,
        long_desc,
    });
    cmd
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;

    let data = Data::load_from_file().await;

    let commands_vec = vec![
        cmd_with_meta(
            help(),
            CommandCategory::Basic,
            "Show help; Use `/help <command>` to get more help",
            "",
        ),
        cmd_with_meta(
            language(),
            CommandCategory::Basic,
            "Set messages language; Available: english, polski",
            "",
        ),
        cmd_with_meta(
            croll(),
            CommandCategory::Basic,
            "Skill test with optional bonus and penalty dice",
            CROLL_HELP,
        ),
        cmd_with_meta(
            hcroll(),
            CommandCategory::Basic,
            "Same as `/croll` but with private output",
            "",
        ),
        cmd_with_meta(
            roll(),
            CommandCategory::Basic,
            "Simple dice roll with optional multiplier and/or modifier",
            ROLL_HELP,
        ),
        cmd_with_meta(
            hroll(),
            CommandCategory::Basic,
            "Same as `/roll` but with private output",
            "",
        ),
        cmd_with_meta(
            initiative(),
            CommandCategory::Basic,
            "Initiative test with optional bonus and penalty dice",
            INITIATIVE_HELP,
        ),
        cmd_with_meta(improve(), CommandCategory::Basic, "Improve skill test", IMPROVE_HELP),
        cmd_with_meta(
            levels(),
            CommandCategory::Basic,
            "Success levels of provided threshold",
            LEVELS_HELP,
        ),
    ];

    #[cfg(feature = "character-sheet")]
    let commands_vec = commands_vec
        .into_iter()
        .chain(vec![
            cmd_with_meta(
                character_cmd(),
                CommandCategory::Character,
                "Manage character entries",
                "create, select, reset, delete",
            ),
            cmd_with_meta(
                skill_cmd(),
                CommandCategory::Character,
                "Skill check or edit",
                "check, mark, improve, add, set, delete",
            ),
            cmd_with_meta(
                attribute(),
                CommandCategory::Character,
                "Attribute check or set value",
                "",
            ),
            cmd_with_meta(
                weapon(),
                CommandCategory::Character,
                "Character's weapons",
                "damage, list, add, delete",
            ),
            cmd_with_meta(
                item(),
                CommandCategory::Character,
                "Character's items",
                "list, add, delete",
            ),
            cmd_with_meta(status(), CommandCategory::Character, "Character's status", ""),
            cmd_with_meta(sheet(), CommandCategory::Character, "Character's sheet", ""),
            cmd_with_meta(hp(), CommandCategory::Character, "Shows/modifies HP", ""),
            cmd_with_meta(
                sanity(),
                CommandCategory::Character,
                "Roll Sanity check or modify Sanity points",
                "",
            ),
            cmd_with_meta(
                luck(),
                CommandCategory::Character,
                "Roll Luck check or modify Luck points",
                "",
            ),
            cmd_with_meta(dodge(), CommandCategory::Character, "Roll Dodge check", ""),
            cmd_with_meta(
                fight(),
                CommandCategory::Character,
                "Roll combat Fighting/Firearms check",
                "",
            ),
            cmd_with_meta(
                damage(),
                CommandCategory::Character,
                "Roll damage for equipped weapon",
                "",
            ),
            cmd_with_meta(magic(), CommandCategory::Character, "Shows/modifies Magic points", ""),
            cmd_with_meta(
                sleep(),
                CommandCategory::Character,
                "Updated initial Sanity level (for tracking daily Sanity loss)",
                "",
            ),
            cmd_with_meta(gmstatus(), CommandCategory::GM, "Show all active characters status", ""),
            cmd_with_meta(
                gmsheet(),
                CommandCategory::GM,
                "Show one of active characters sheet",
                "",
            ),
            cmd_with_meta(
                gmcharacter_cmd(),
                CommandCategory::GM,
                "GM API for players characters",
                "",
            ),
            cmd_with_meta(gmhp(), CommandCategory::GM, "Modify one of active characters HP", ""),
            cmd_with_meta(
                gmsanity(),
                CommandCategory::GM,
                "Modify one of active characters Sanity",
                "",
            ),
            cmd_with_meta(kill(), CommandCategory::GM, "Kill one of active characters", ""),
            cmd_with_meta(revive(), CommandCategory::GM, "Revive one of active characters", ""),
            cmd_with_meta(
                heal(),
                CommandCategory::GM,
                "Heal (remove Major Wound) one of active characters",
                "",
            ),
            cmd_with_meta(
                insane(),
                CommandCategory::GM,
                "Make insane (add Fragile Mind) one of active characters",
                "",
            ),
            cmd_with_meta(
                sane(),
                CommandCategory::GM,
                "Make sane (remove Fragile Mind) one of active characters",
                "",
            ),
            cmd_with_meta(db(), CommandCategory::GM, "Download/Upload database", ""),
            cmd_with_meta(quicksave(), CommandCategory::GM, "Create quick backup of database", ""),
            cmd_with_meta(quickload(), CommandCategory::GM, "Load quick backup of database", ""),
        ])
        .collect();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands_vec,
            on_error: |error| Box::pin(handle_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let mut client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await?;

    Ok(client.start().await?)
}

async fn handle_error(error: FrameworkError<'_>) {
    match error {
        poise::FrameworkError::Command { ctx, error, .. } => {
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
            let _ = ctx
                .send(
                    poise::CreateReply::default()
                        .content("🚫 This command is only available to GMs.")
                        .ephemeral(true),
                )
                .await;
        }
        _ => (),
    }
}
