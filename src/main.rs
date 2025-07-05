use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents},
    CreateReply,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use tokio::{fs, sync::RwLock};
mod roll;
use roll::{
    improve_skill, roll_dice, roll_skill, CharacterInitiative, DiceResult, InitiativeResult,
    SkillResult,
};
mod message;
use message::{format_dice, format_improve, format_initiative, format_levels, format_skill};
mod character;
use character::Character;

use crate::message::Message;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserData {
    pub characters: HashMap<String, Character>,
    pub active_character: Option<String>,
}

pub type UserId = u64;
pub type UsersHashMap = HashMap<UserId, UserData>;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InnerData {
    pub users: UsersHashMap,
    pub max_characters_per_user: usize,
}

struct Data {
    pub data: RwLock<InnerData>,
    pub save_path: String,
}

impl Data {
    pub async fn load_from_file(path: &str) -> Self {
        match fs::read_to_string(path).await {
            Ok(content) => {
                if let Ok(inner_data) = serde_json::from_str::<InnerData>(&content) {
                    Data {
                        data: RwLock::new(inner_data),
                        save_path: path.to_string(),
                    }
                } else {
                    println!("Failed to parse JSON, starting fresh.");
                    Data::empty(path)
                }
            }
            Err(_) => {
                println!("No save file found, starting fresh.");
                Data::empty(path)
            }
        }
    }

    pub fn empty(path: &str) -> Self {
        Data {
            data: RwLock::new(InnerData {
                users: HashMap::new(),
                max_characters_per_user: 5,
            }),
            save_path: path.to_string(),
        }
    }

    pub async fn save_to_file(&self) -> Result<(), Error> {
        let inner_data;
        {
            inner_data = self.data.read().await.clone();
        }
        fs::write(&self.save_path, serde_json::to_string_pretty(&inner_data)?).await?;
        Ok(())
    }
}

impl Display for InnerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            serde_json::to_string_pretty(&self)
                .map_err(|_| std::fmt::Error)?
                .as_str(),
        )
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
type FrameworkError<'a> = poise::FrameworkError<'a, Data, Error>;

async fn is_user_gm(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let gm_role_id;
        {
            let guild = guild_id
                .to_guild_cached(ctx.serenity_context())
                .ok_or("Guild not in cache")?;

            gm_role_id = guild
                .role_by_name("GM")
                .ok_or("No `GM` role".to_string())?
                .id
                .get();
        }
        if let Ok(member) = guild_id
            .member(ctx.serenity_context(), ctx.author().id)
            .await
        {
            let has_role = member.roles.iter().any(|r| r.get() == gm_role_id);
            return Ok(has_role);
        }
    }

    Ok(false)
}

/// Use `/help <command>` to get more help.
#[poise::command(slash_command, track_edits)]
pub async fn help(ctx: Context<'_>, command: Option<String>) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

/// Skill test with optional bonus and penalty dice
///
/// Bonus ('+' or 'b') and penalty ('-', 'p' or 'k') dice are being resolved automatically for easier adding circumstances of the roll, for example: test you firearms skill test of threshold 70, you've been aiming entire previous round (bonus), target is really big (bonus) but moving fast (penalty) so you can roll 70++-.
///
/// Syntax: `<threshold>` `<optional modifier dice symbols>...`
///
/// Examples:
/// `30+`, `20--`, `50`, `50p`, `50k`, `70bb`, `20bbppp`, `40bk`
///
/// `/croll 60ppb` results with:
/// ```
/// Success
/// 38
/// Rolls: [ 30 ] [ 20 ] [ 8 ]
/// 8 points to Hard Success
/// Threshold: 60 / 30 / 12
/// Penalty dice: 1
/// Query: "60ppb"
/// ```
#[poise::command(slash_command, track_edits)]
async fn croll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    ctx.send(CreateReply::default().embed(format_skill(
        threshold.clone(),
        croll_impl(threshold)?,
        true,
    )))
    .await?;
    Ok(())
}

/// Same as `/croll` but with hidden output
#[poise::command(slash_command, track_edits)]
async fn hcroll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    ctx.send(CreateReply::default().ephemeral(true).embed(format_skill(
        threshold.clone(),
        croll_impl(threshold)?,
        true,
    )))
    .await?;
    Ok(())
}

fn croll_impl(query: String) -> Result<SkillResult, Error> {
    let pattern = r"^(\d+)([bpk\+-]*)$";
    let re = Regex::new(pattern).unwrap();
    let query = query.replace(' ', "");
    let captures = re
        .captures(&query)
        .ok_or(format!("Invalid query: \"{query}\""))?;
    let threshold = captures
        .get(1)
        .ok_or("Invalid threshold:")?
        .as_str()
        .parse()?;
    let mut penalty = 0;
    let mut bonus = 0;
    match captures.get(2) {
        None => (),
        Some(captures_match) => {
            let penalty_bonus_str = captures_match.as_str();
            penalty = penalty_bonus_str
                .chars()
                .filter(|c| *c == 'p' || *c == 'k' || *c == '-')
                .count() as i32;
            bonus = penalty_bonus_str
                .chars()
                .filter(|c| *c == 'b' || *c == '+')
                .count() as i32;
        }
    }
    Ok(roll_skill(threshold, penalty, bonus))
}

/// Improve skill test
///
/// Syntax: `<threshold>`
///
/// `/improve 60` results with:
/// ```
/// Success
/// 67
/// Threshold: 60
/// Query: "60"
/// ```
#[poise::command(slash_command, track_edits)]
async fn improve(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    let pattern = r"^\D*(\d+)\D*$";
    let re = Regex::new(pattern).unwrap();
    let threshold_stripped = threshold.replace(' ', "");
    let captures = re
        .captures(&threshold_stripped)
        .ok_or(format!("Invalid query: \"{threshold_stripped}\""))?;
    let threshold_int = captures
        .get(1)
        .ok_or("Invalid threshold:")?
        .as_str()
        .parse()?;
    ctx.send(CreateReply::default().embed(format_improve(
        threshold.clone(),
        improve_skill(threshold_int),
    )))
    .await?;
    Ok(())
}

/// Simple dice roll with optional multiplier and/or modifier
///
/// Syntax: `<optional number of dice>` `d/k` `<sides>` `<optional multiplier>` `<optional modifier>`
///
/// Examples:
/// `2d4`, `3k6`, `24k6+10`, `12d8x3`, `4k12*2`, `6d6x6+6`
///
/// `/roll 3d6x5+1` results with:
/// ```
/// 71
/// Rolls: ( [ 5 ] [ 6 ] [ 3 ] ) x 5 + 1
/// Query: "3d6x5+1"
/// ```
#[poise::command(slash_command, track_edits)]
async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    ctx.send(CreateReply::default().embed(format_dice(dice.clone(), roll_impl(dice)?, true)))
        .await?;
    Ok(())
}

/// Same as `/roll` but with hidden output
#[poise::command(slash_command, track_edits)]
async fn hroll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .embed(format_dice(dice.clone(), roll_impl(dice)?, true))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

fn roll_impl(dice: String) -> Result<DiceResult, Error> {
    let pattern = r"^(\d+)?[kd](\d+)([x\*](\d+))?([+-]\d+)?$";
    let re = Regex::new(pattern).unwrap();
    let dice_stripped = dice.replace(' ', "");
    let captures = re
        .captures(&dice_stripped)
        .ok_or(format!("Invalid query: \"{dice_stripped}\""))?;
    let dice_count = match captures.get(1) {
        Some(m) => m.as_str().parse()?,
        None => 1,
    };
    let sides = captures
        .get(2)
        .ok_or("No dice type")?
        .as_str()
        .parse::<i32>()?;
    let multiplier = match captures.get(3) {
        Some(_) => match captures.get(4) {
            Some(m) => Some(m.as_str().parse()?),
            None => None,
        },
        None => None,
    };
    let modifier = match captures.get(5) {
        Some(m) => Some(m.as_str().parse()?),
        None => None,
    };
    Ok(roll_dice(dice_count, sides, modifier, multiplier))
}

/// Initiative test with optional bonus and penalty dice
///
/// Initiative order is defined by: dexterity test success level, dexterity value and lowest roll value.
///
/// Bonus and penalty dice are being resolved automatically for easier adding circumstances of the roll, for example: you gain bonus die for initiative roll for being prepared (armed) at the beginning of fight. (see `/croll` command)
///
/// Syntax: `<character_name> <dexterity> <character_name> <dexterity> ...`
///
/// For example `/initiative Anna 50 Brian 60 Celine 60 Douglas 70 Emma 50 Frank 50 George 50` results with:
/// ```
/// Initiative
/// 1. Douglas (Success) [Dex:70 Roll:62]
/// 2. Celine (Success) [Dex:60 Roll:36]
/// 3. Brian (Success) [Dex:60 Roll:43]
/// 4. Emma (Success) [Dex:50 Roll:50]
/// 5. Anna (Failure) [Dex:50 Roll:51]
/// 6. Frank (Failure) [Dex:50 Roll:65]
/// 7. George (Failure) [Dex:50 Roll:91]
/// Query: "Anna 50 Brian 60 Celine 60 Douglas 70 Emma 50 Frank 50 George 50"
/// ```
#[poise::command(slash_command, track_edits)]
async fn initiative(ctx: Context<'_>, input: String) -> Result<(), Error> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() % 2 != 0 {
        ctx.send(
            CreateReply::default()
                .embed(
                    Message {
                        description: format!(
                            "Query must contain pairs of `Name` and `Dex` thresholds: \"{}\"",
                            input
                        ),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }
    let mut characters: Vec<CharacterInitiative> = vec![];
    for pair in words.chunks(2) {
        let name = pair[0];
        let threshold = pair[1];
        let skill_result = croll_impl(threshold.to_string())?;
        characters.push(CharacterInitiative {
            result: skill_result,
            name: name.to_string(),
        });
    }

    let initiative_result = InitiativeResult::new(characters);
    ctx.send(CreateReply::default().embed(format_initiative(
        input.clone(),
        initiative_result.clone(),
        false,
    )))
    .await?;
    ctx.send(
        CreateReply::default()
            .embed(format_initiative(
                input.clone(),
                initiative_result.clone(),
                true,
            ))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Success levels of provided threshold
///
/// Syntax: `<threshold>`
///
/// `/levels 50` results with:
/// ```
/// 50 / 25 / 10
/// Threshold: 50
/// Query: "50"
/// ```
#[poise::command(slash_command, track_edits)]
async fn levels(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    let pattern = r"^\D*(\d+)\D*$";
    let re = Regex::new(pattern).unwrap();
    let threshold_stripped = threshold.replace(' ', "");
    let captures = re
        .captures(&threshold_stripped)
        .ok_or(format!("Invalid query: \"{threshold_stripped}\""))?;
    let threshold_int = captures
        .get(1)
        .ok_or("Invalid threshold:")?
        .as_str()
        .parse()?;
    ctx.send(
        CreateReply::default()
            .embed(format_levels(threshold.clone(), threshold_int))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

async fn autocomplete_my_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    let user_id = ctx.author().id;

    data.users
        .get(&user_id.get())
        .into_iter()
        .flat_map(|user| user.characters.keys())
        .filter(move |name| name.to_lowercase().starts_with(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect()
}

async fn autocomplete_any_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    data.users
        .values()
        .flat_map(|user| user.characters.keys())
        .filter(|name| name.to_lowercase().starts_with(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect()
}

async fn autocomplete_any_active_character<'a>(ctx: Context<'a>, partial: &'a str) -> Vec<String> {
    let data = ctx.data().data.read().await.clone();
    data.users
        .values()
        .filter_map(move |user| user.active_character.as_ref())
        .filter(|name| name.to_lowercase().starts_with(&partial.to_lowercase()))
        .take(25) // Discord's limit
        .cloned()
        .collect()
}

/// Create character
#[poise::command(slash_command)]
async fn character_create(
    ctx: Context<'_>,
    #[description = "Name"] name: String,
    #[description = "HP"] hp: i32,
    #[description = "Sanity"] sanity: i32,
    #[description = "Luck"] luck: i32,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let max = data.max_characters_per_user;
        let user_entry = data.users.entry(user_id).or_default();

        if user_entry.characters.len() >= max {
            message.title = "Sorry, you have too many characters already".to_string();
            ephemeral = true;
        } else {
            if user_entry
                .characters
                .insert(name.clone(), Character::new(name.clone(), hp, sanity, luck))
                .is_some()
            {
                message.title = format!("✅ `{}` updated.", name)
            } else {
                message.title = format!("✅ `{}` created.", name)
            }
            message.description = format!(
                "❤️ HP: **{}**, 🧠 Sanity: **{}**, 🍀 Luck: **{}**",
                hp, sanity, luck
            );
            user_entry.active_character = Some(name);
        }
    }

    ctx.send(
        CreateReply::default()
            .embed(message.to_embed())
            .ephemeral(ephemeral),
    )
    .await?;

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Delete character
#[poise::command(slash_command)]
async fn character_delete(
    ctx: Context<'_>,
    #[description = "Character to delete"]
    #[autocomplete = "autocomplete_my_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;

        user_data.characters.remove(&character_name);

        if let Some(active_character) = &mut user_data.active_character {
            if active_character == &character_name {
                user_data.active_character = None;
            }
        }
    }

    ctx.send(
        CreateReply::default().embed(
            Message {
                title: format!("❎ `{}` deleted.", character_name),
                ..Default::default()
            }
            .to_embed(),
        ),
    )
    .await?;

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Deletes character [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn gmcharacter_delete(
    ctx: Context<'_>,
    #[description = "Character to delete"]
    #[autocomplete = "autocomplete_any_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if user_data.characters.get_mut(&character_name).is_some() {
                user_data.characters.remove(&character_name);
                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("❎ `{}` deleted.", character_name),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Select character to play
#[poise::command(slash_command)]
async fn character_select(
    ctx: Context<'_>,
    #[description = "Character to select"]
    #[autocomplete = "autocomplete_my_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        user_data.active_character = Some(character.name.clone());
    }

    ctx.send(
        CreateReply::default()
            .embed(
                Message {
                    title: format!("`{}` selected.", &character_name),
                    ..Default::default()
                }
                .to_embed(),
            )
            .ephemeral(true),
    )
    .await?;

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Resets active character stats to initial values
#[poise::command(slash_command)]
async fn character_reset(ctx: Context<'_>) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        character.reset();

        ctx.send(
            CreateReply::default().embed(
                Message {
                    title: format!("`{}` has been revived with full strength. 💪", &active),
                    ..Default::default()
                }
                .to_embed(),
            ),
        )
        .await?;
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Resets active character stats to initial values
#[poise::command(slash_command, check = "is_user_gm")]
async fn gmcharacter_reset(
    ctx: Context<'_>,
    #[description = "Character to delete"]
    #[autocomplete = "autocomplete_any_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                character.fragile_mind = true;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "`{}` has been revived with full strength. 💪",
                                &character.name
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Shows active character status
#[poise::command(slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let data = ctx.data().data.read().await;
    let user_data = data.users.get(&user_id).ok_or("No characters.")?;
    let active = user_data
        .active_character
        .clone()
        .ok_or("No active character.")?;
    let character = user_data
        .characters
        .get(&active)
        .ok_or(format!("Character `{}` not found.", &active))?;

    let message = Message {
        title: format!("`{}`'s status", character.name),
        description: character.status(),
        ..Default::default()
    };

    ctx.send(
        CreateReply::default()
            .embed(message.to_embed())
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Show all active characters' status [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn gmstatus(ctx: Context<'_>) -> Result<(), Error> {
    let mut message = Message {
        title: "Characters' status".to_string(),
        ..Default::default()
    };
    let data = ctx.data().data.read().await.clone();

    if data.users.is_empty() {
        message.description = "No active characters.".to_string();
    } else {
        for (_, user_data) in data.users.clone().into_iter() {
            if let Some(active) = &user_data.active_character {
                if let Some(character) = user_data.characters.get(active) {
                    message.description =
                        format!("{}\n{}", message.description, character.status_oneline());
                }
            }
        }
    }

    ctx.send(
        CreateReply::default()
            .embed(message.to_embed())
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Sets "Fragile Mind" status for active character [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn insane(
    ctx: Context<'_>,
    #[description = "Character to become mad"]
    #[autocomplete = "autocomplete_any_active_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                character.fragile_mind = true;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("**`{}`'s gone mad!**", character.name),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Clears "Fragile Mind" status for active character [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn sane(
    ctx: Context<'_>,
    #[description = "Character to become sane"]
    #[autocomplete = "autocomplete_any_active_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                character.fragile_mind = false;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "**`{}`'s mind healed enough to carry on...**",
                                character.name
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Clears "Major Wound" status for active character [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn heal(
    ctx: Context<'_>,
    #[description = "Character to heal"]
    #[autocomplete = "autocomplete_any_active_character"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                character.major_wound = false;

                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!(
                                "**`{}`'s body healed enough to carry on...**",
                                character.name
                            ),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Sleep/long rest event for active character
#[poise::command(slash_command)]
async fn sleep(ctx: Context<'_>) -> Result<(), Error> {
    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        character.sanity.update_initial();

        ctx.send(
            CreateReply::default().embed(
                Message {
                    title: format!("`{}` has survived another day. 🎉", character.name),
                    description: format!(
                        "Updating initial 🧠 Sanity: **{}**/{}",
                        character.sanity.current, character.sanity.initial
                    ),
                    ..Default::default()
                }
                .to_embed(),
            ),
        )
        .await?;
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Kill character [GM only]
#[poise::command(slash_command, check = "is_user_gm")]
async fn kill(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_any_active_character"]
    #[description = "Character name to kill"]
    character_name: String,
) -> Result<(), Error> {
    {
        let mut data = ctx.data().data.write().await;

        for user_data in data.users.values_mut() {
            if let Some(character) = user_data.characters.get_mut(&character_name) {
                character.hp.current = 0;
                character.dead = true;
                ctx.send(
                    CreateReply::default().embed(
                        Message {
                            title: format!("**`{}` has died.**", &character_name),
                            ..Default::default()
                        }
                        .to_embed(),
                    ),
                )
                .await?;
                break;
            }
        }
    }

    ctx.data().save_to_file().await?;
    Ok(())
}

/// Shows/modifies current HPs for active character
#[poise::command(slash_command)]
async fn hp(
    ctx: Context<'_>,
    #[description = "Amount to modify HP by"] delta: Option<String>,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            character.hp.modify(delta);

            if delta < -(character.hp.max) {
                character.dead = true;
                character.major_wound = false;
                message.description = "**Death is inevitable.**".to_string();
            } else {
                if delta >= (character.hp.max / 2) && character.major_wound {
                    character.major_wound = false;
                    message.description = "**Major wound healed!**".to_string();
                }
                if delta <= -(character.hp.max / 2) {
                    character.major_wound = true;
                    message.description = "**Major wound!**".to_string();
                    if character.hp.current > 0 {
                        message.description = format!(
                            "{}\n{}",
                            message.description, "You fell.\nRoll a **CON** test not to blackout."
                        )
                        .to_string();
                    }
                }

                if character.hp.current == 0 {
                    if character.major_wound {
                        message.description =format!(
                                        "{}\n{}",
                                        message.description,
                            "**Agony!**\nYou fell.\nYou blackout.\nRoll a **CON** test not to **die**.\nYou'll be rolling this every round until someone helps you...");
                    } else {
                        message.description = format!(
                            "{}\n{}",
                            message.description, "**Knock Out!**\nYou fell.\nYou blackout.",
                        );
                    }
                }
            }

            message.title = format!(
                "{}\n`{}`'s ❤️ HP modified by **{:+}**. {}",
                message.title,
                character.name,
                delta,
                character.status_hp()
            );
        } else {
            message.title = character.status_hp();
            ephemeral = true;
        }
        ctx.send(
            CreateReply::default()
                .embed(message.to_embed())
                .ephemeral(ephemeral),
        )
        .await?;
    }

    if delta.is_some() {
        ctx.data().save_to_file().await?;
    }
    Ok(())
}

/// Shows/modifies current Sanity points for active character
#[poise::command(slash_command)]
async fn sanity(
    ctx: Context<'_>,
    #[description = "Amount to modify Sanity by"] delta: Option<String>,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            character.sanity.modify(delta);

            if character.sanity.current == 0 {
                character.insane = true;
                message.description = "**Your mind has been irreversably shattered.**".to_string();
            } else if character.fragile_mind && delta < 0 {
                message.description = "**Temporal insanity!**".to_string();
            } else if delta <= -5 {
                message.description = "**Temporal insanity threat!**\nRoll an **INT** check if you **really** understood what just happened.".to_string();
            } else if (character.sanity.current as f32 / character.sanity.initial as f32) < 0.8 {
                character.fragile_mind = true;
                message.description = "**Indefinite insanity!**".to_string();
            }

            message.title = format!(
                "`{}`'s 🧠 Sanity modified by **{:+}**. {}",
                character.name,
                delta,
                character.status_sanity()
            );
        } else {
            message.title = character.status_sanity();
            ephemeral = true;
        }

        ctx.send(
            CreateReply::default()
                .embed(message.to_embed())
                .ephemeral(ephemeral),
        )
        .await?;
    }

    if delta.is_some() {
        ctx.data().save_to_file().await?;
    }
    Ok(())
}

/// Shows/modifies current Luck points for active character
#[poise::command(slash_command)]
async fn luck(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Amount to modify Luck by"] delta: Option<String>,
) -> Result<(), Error> {
    let mut message = Message::default();
    let mut ephemeral = false;

    {
        let user_id = ctx.author().id.get();
        let mut data = ctx.data().data.write().await;
        let user_data = data.users.get_mut(&user_id).ok_or("No characters.")?;
        let active = user_data
            .active_character
            .clone()
            .ok_or("No active character.")?;
        let character = user_data
            .characters
            .get_mut(&active)
            .ok_or(format!("Character `{}` not found.", &active))?;

        if let Some(delta) = &delta {
            let delta = delta.replace(' ', "").parse::<i32>()?;
            character.luck.modify(delta);

            message.title = format!(
                "`{}`'s 🍀 Luck modified by **{:+}**. Luck: **{}**",
                character.name, delta, character.luck.current
            );
        } else {
            message.title = format!(
                "`{}`'s 🍀 Luck: **{}**",
                character.name, character.luck.current
            );
            ephemeral = true;
        }
    }

    ctx.send(
        CreateReply::default()
            .embed(message.to_embed())
            .ephemeral(ephemeral),
    )
    .await?;

    if delta.is_some() {
        ctx.data().save_to_file().await?;
    }
    Ok(())
}

/// Database get/set [GM only]
#[poise::command(prefix_command, slash_command, check = "is_user_gm")]
async fn db(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "DB in JSON format"] db: Option<String>,
) -> Result<(), Error> {
    if let Some(db) = db {
        let db_parsed = serde_json::from_str::<InnerData>(&db)?;

        {
            let mut inner_data;
            inner_data = ctx.data().data.write().await;
            inner_data.clone_from(&db_parsed);
        }

        ctx.data().save_to_file().await?;

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
        let inner_data;
        {
            inner_data = ctx.data().data.read().await.clone();
        }

        ctx.send(
            CreateReply::default()
                .embed(
                    Message {
                        description: inner_data.to_string(),
                        ..Default::default()
                    }
                    .to_embed(),
                )
                .ephemeral(true),
        )
        .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let discord_token = std::env::var("DISCORD_TOKEN")?;

    let data = Data::load_from_file("db.json").await;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                croll(),
                hcroll(),
                roll(),
                hroll(),
                improve(),
                initiative(),
                levels(),
                character_create(),
                character_delete(),
                character_select(),
                character_reset(),
                status(),
                hp(),
                sanity(),
                luck(),
                sleep(),
                gmstatus(),
                gmcharacter_reset(),
                gmcharacter_delete(),
                kill(),
                heal(),
                insane(),
                sane(),
                db(),
            ],
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
                        .content(format!("**{error}**\n\n{help_message}"))
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
