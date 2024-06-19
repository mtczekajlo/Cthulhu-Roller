use anyhow::Context as _;
use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents},
    CreateReply,
};
use regex::Regex;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod roll;
use roll::{roll_dice, roll_skill};

mod message;
use message::{format_dice, format_skill};

#[poise::command(slash_command)]
async fn croll(ctx: Context<'_>, threshold: String) -> Result<(), Error> {
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
    let roll_result = roll_skill(threshold_int, penalty, bonus);
    ctx.send(CreateReply::default().embed(format_skill(threshold, roll_result)))
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
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

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![croll(), roll()],
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
