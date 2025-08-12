use crate::bot_data::ContextData;
use crate::commands::basic::roll_cmd;
#[cfg(feature = "character-sheet")]
use crate::commands::character::fight::fight_cmd;
#[cfg(feature = "character-sheet")]
use crate::commands::{
    character::sheet_cmd,
    gm::character::{
        gmheal_cmd, gmhp_cmd, gminsane_cmd, gmkill_cmd, gmrevive_cmd, gmsan_cmd, gmsane_cmd, gmsheet_cmd, gmstatus_cmd,
        gmwound_cmd,
    },
};
#[cfg(feature = "character-sheet")]
use crate::commands::{
    character::{attribute::attribute_cmd, character_cmd::character_cmd, skill::skill_cmd},
    gm::db::{gmdatabase_cmd, gmquickload_cmd, gmquicksave_cmd},
};
#[cfg(feature = "character-sheet")]
use crate::commands::{
    character::{fight::damage_cmd, weapon::weapon_cmd},
    gm::{characters::gmcharacter_cmd, item::gmitem_cmd, weapon::gmweapon_cmd},
};
use crate::help;
use crate::types::Error;
use poise::Command;

#[cfg(feature = "character-sheet")]
use crate::commands::character::{
    item::item_cmd,
    skills::{dodge_cmd, listen_cmd, maneuver_cmd, spot_hidden_cmd},
    sleep_cmd,
    stats::{hp_cmd, luck_cmd, mp_cmd, sanity_cmd},
    status_cmd,
};
use crate::{
    commands::basic::{
        croll_cmd, end_battle_cmd, hcroll_cmd, hroll_cmd, improve_test_cmd, initiative_cmd, language_cmd, levels_cmd,
        next_round_cmd, previous_round_cmd, roll_attributes_cmd,
    },
    message::help::{CROLL_HELP, IMPROVE_HELP, INITIATIVE_HELP, LEVELS_HELP, ROLL_HELP},
};

#[allow(dead_code)]
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

fn cmd_with_meta(
    mut cmd: poise::Command<ContextData, Error>,
    category: CommandCategory,
    short_desc: &'static str,
    long_desc: &'static str,
) -> poise::Command<ContextData, Error> {
    cmd.custom_data = Box::new(CommandMeta {
        category,
        short_desc,
        long_desc,
    });
    cmd
}

pub fn command_list() -> Vec<Command<ContextData, Error>> {
    let commands_vec = vec![
        cmd_with_meta(
            help(),
            CommandCategory::Basic,
            "Show help; Use `/help <command>` to get more help",
            "",
        ),
        cmd_with_meta(
            language_cmd(),
            CommandCategory::Basic,
            "Set messages language; Available: `english`, `polski`",
            "",
        ),
        cmd_with_meta(
            croll_cmd(),
            CommandCategory::Basic,
            "Skill test with optional bonus and penalty dice",
            CROLL_HELP,
        ),
        cmd_with_meta(
            hcroll_cmd(),
            CommandCategory::Basic,
            "Same as `/croll` but with hidden details",
            "",
        ),
        cmd_with_meta(
            roll_cmd(),
            CommandCategory::Basic,
            "Simple dice roll with optional multiplier and/or modifier",
            ROLL_HELP,
        ),
        cmd_with_meta(
            hroll_cmd(),
            CommandCategory::Basic,
            "Same as `/roll` but with hidden details",
            "",
        ),
        cmd_with_meta(
            initiative_cmd(),
            CommandCategory::Basic,
            "Initiative test with optional bonus and penalty dice; Start battle",
            INITIATIVE_HELP,
        ),
        cmd_with_meta(next_round_cmd(), CommandCategory::Basic, "Next battle round", ""),
        cmd_with_meta(
            previous_round_cmd(),
            CommandCategory::Basic,
            "Previous battle round",
            "",
        ),
        cmd_with_meta(end_battle_cmd(), CommandCategory::Basic, "End battle", ""),
        cmd_with_meta(
            improve_test_cmd(),
            CommandCategory::Basic,
            "Improve skill test",
            IMPROVE_HELP,
        ),
        cmd_with_meta(
            levels_cmd(),
            CommandCategory::Basic,
            "Success levels of provided threshold",
            LEVELS_HELP,
        ),
        cmd_with_meta(
            roll_attributes_cmd(),
            CommandCategory::Basic,
            "Roll attributes (characteristics) for character creation",
            "",
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
                "",
            ),
            cmd_with_meta(
                attribute_cmd(),
                CommandCategory::Character,
                "Attribute (Characteristic) check or set value",
                "",
            ),
            cmd_with_meta(skill_cmd(), CommandCategory::Character, "Skill check or edit", ""),
            cmd_with_meta(weapon_cmd(), CommandCategory::Character, "Character's weapons", ""),
            cmd_with_meta(item_cmd(), CommandCategory::Character, "Character's items", ""),
            cmd_with_meta(status_cmd(), CommandCategory::Character, "Character's status", ""),
            cmd_with_meta(sheet_cmd(), CommandCategory::Character, "Character's sheet", ""),
            cmd_with_meta(
                hp_cmd(),
                CommandCategory::Character,
                "Modify character's Hit Points",
                "",
            ),
            cmd_with_meta(
                sanity_cmd(),
                CommandCategory::Character,
                "Roll Sanity check or modify Sanity Points",
                "",
            ),
            cmd_with_meta(
                luck_cmd(),
                CommandCategory::Character,
                "Roll Luck check or modify Luck Points",
                "",
            ),
            cmd_with_meta(
                spot_hidden_cmd(),
                CommandCategory::Character,
                "Roll Spot Hidden check",
                "",
            ),
            cmd_with_meta(listen_cmd(), CommandCategory::Character, "Roll Listen check", ""),
            cmd_with_meta(dodge_cmd(), CommandCategory::Character, "Roll Dodge check", ""),
            cmd_with_meta(
                maneuver_cmd(),
                CommandCategory::Character,
                "Roll Maneuver (Fighting (Brawl)) check",
                "",
            ),
            cmd_with_meta(
                fight_cmd(),
                CommandCategory::Character,
                "Roll combat Fighting/Firearms check",
                "",
            ),
            cmd_with_meta(
                damage_cmd(),
                CommandCategory::Character,
                "Roll damage for equipped weapon",
                "",
            ),
            cmd_with_meta(
                mp_cmd(),
                CommandCategory::Character,
                "Modify character's Magic Points",
                "",
            ),
            cmd_with_meta(
                sleep_cmd(),
                CommandCategory::Character,
                "Update initial Sanity level (for tracking daily Sanity loss)",
                "",
            ),
            cmd_with_meta(
                gmstatus_cmd(),
                CommandCategory::GM,
                "Show all active characters status",
                "",
            ),
            cmd_with_meta(
                gmsheet_cmd(),
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
            cmd_with_meta(
                gmhp_cmd(),
                CommandCategory::GM,
                "Modify one of active characters HP",
                "",
            ),
            cmd_with_meta(
                gmsan_cmd(),
                CommandCategory::GM,
                "Modify one of active characters Sanity",
                "",
            ),
            cmd_with_meta(gmkill_cmd(), CommandCategory::GM, "Kill one of active characters", ""),
            cmd_with_meta(
                gmrevive_cmd(),
                CommandCategory::GM,
                "Revive one of active characters",
                "",
            ),
            cmd_with_meta(
                gmwound_cmd(),
                CommandCategory::GM,
                "Add Major Wound to one of active characters",
                "",
            ),
            cmd_with_meta(
                gmheal_cmd(),
                CommandCategory::GM,
                "Remove Major Wound from one of active characters",
                "",
            ),
            cmd_with_meta(
                gminsane_cmd(),
                CommandCategory::GM,
                "Add Fragile Mind to one of active characters",
                "",
            ),
            cmd_with_meta(
                gmsane_cmd(),
                CommandCategory::GM,
                "Remove Fragile Mind from one of active characters",
                "",
            ),
            cmd_with_meta(
                gmweapon_cmd(),
                CommandCategory::GM,
                "GM API for active character weapons",
                "",
            ),
            cmd_with_meta(
                gmitem_cmd(),
                CommandCategory::GM,
                "GM API for active character items",
                "",
            ),
            cmd_with_meta(gmdatabase_cmd(), CommandCategory::GM, "Download/Upload database", ""),
            cmd_with_meta(
                gmquicksave_cmd(),
                CommandCategory::GM,
                "Create quick backup of database",
                "",
            ),
            cmd_with_meta(
                gmquickload_cmd(),
                CommandCategory::GM,
                "Load quick backup of database",
                "",
            ),
        ])
        .collect();
    commands_vec
}
