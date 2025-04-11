use anyhow::Context as _;
use poise::{
    serenity_prelude::{
        futures::{self, Stream, StreamExt},
        ClientBuilder, GatewayIntents,
    },
    CreateReply,
};
use regex::Regex;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod roll;
use roll::{roll_dice, roll_skill, Character, InitiativeResult, SkillResult};

mod message;
use message::{format_dice, format_initiative, format_skill};

async fn autocomplete_help<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    futures::stream::iter(&["croll", "roll", "initiative"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

#[poise::command(slash_command, track_edits)]
/// Use `/help <command>` to get more help.
pub async fn help(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_help"] command: Option<String>,
) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
/// Call of Cthulhu 7E skill test roller with optional bonus and penalty dice.
///
/// Bonus and penalty dice are being resolved automatically for easier adding circumstances of the roll, for example: test you firearms skill test of threshold 70, you've been aiming entire previous round (bonus), target is really big (bonus) but moving fast (penalty) so you can roll 70bbp.
///
/// Syntax: `<threshold>` `<bonus die>/<penalty die>...`
///
/// Examples:
/// `50`, `50p`, `70bb`, `20bbppp`
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
async fn croll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
    ctx.send(CreateReply::default().embed(format_skill(threshold.clone(), croll_impl(threshold)?)))
        .await?;
    Ok(())
}

fn croll_impl(threshold: String) -> Result<SkillResult, Error> {
    let pattern = r"^(\d+)([bp]+)?$";
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
    let mut penalty = 0;
    let mut bonus = 0;
    match captures.get(2) {
        None => (),
        Some(captures_match) => {
            let penalty_bonus_str = captures_match.as_str();
            penalty = penalty_bonus_str.chars().filter(|c| *c == 'p').count() as i32;
            bonus = penalty_bonus_str.chars().filter(|c| *c == 'b').count() as i32;
        }
    }
    Ok(roll_skill(threshold_int, penalty, bonus))
}

#[poise::command(slash_command, track_edits)]
/// Generic dice roller with optional multiplier and/or modifier.
///
/// Syntax: `<optional number of dice>` `d/k` `<sides>` `<optional multiplier>` `<optional modifier>`
///
/// Examples:
/// `2d4`, `3k6`, `24k6+10`, `12d8x3`, `6d6x6+6`
///
/// `/roll 3d6x5+1` results with:
/// ```
/// 71
/// Rolls: ( [ 5 ] [ 6 ] [ 3 ] ) x 5 + 1
/// Query: "3d6x5+1"
/// ```
async fn roll(ctx: Context<'_>, dice: String) -> Result<(), Error> {
    let pattern = r"^(\d+)?[kd](\d+)(x\d+)?([+-]\d+)?$";
    let re = Regex::new(pattern).unwrap();
    let dice_stripped = dice.replace(' ', "");
    let captures = re
        .captures(&dice_stripped)
        .ok_or(format!("Invalid query: \"{dice_stripped}\""))?;
    println!("{:?}", captures);
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
        Some(m) => Some(m.as_str().replace('x', "").parse()?),
        None => None,
    };
    let modifier = match captures.get(4) {
        Some(m) => Some(m.as_str().parse()?),
        None => None,
    };
    let roll_result = roll_dice(dice_count, sides, modifier, multiplier);
    ctx.send(CreateReply::default().embed(format_dice(dice, roll_result)))
        .await?;
    Ok(())
}

#[poise::command(slash_command, track_edits)]
/// Call of Cthulhu 7E initiative test roller with optional bonus and penalty dice.
///
/// Initiative order is defined by dexterity test success level, dexterity value and lowest roll value.
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
async fn initiative(ctx: Context<'_>, input: String) -> Result<(), Error> {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() % 2 != 0 {
        ctx.say(format!(
            "Query must contain pairs of name and dexterity thresholds: \"{}\"",
            input
        ))
        .await?;
        return Ok(());
    }
    let mut characters: Vec<Character> = vec![];
    for pair in words.chunks(2) {
        let name = pair[0];
        let threshold = pair[1];
        let skill_result = croll_impl(threshold.to_string())?;
        characters.push(Character {
            result: skill_result,
            name: name.to_string(),
        });
    }
    ctx.send(
        CreateReply::default().embed(format_initiative(input, InitiativeResult::new(characters))),
    )
    .await?;
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![croll(), roll(), initiative(), help()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = ClientBuilder::new(discord_token, GatewayIntents::non_privileged())
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
