pub mod character;
pub mod characters;
pub mod db;
pub mod item;
pub mod weapon;

use crate::types::{Context, Error};

pub async fn is_user_gm(ctx: Context<'_>) -> Result<bool, Error> {
    if let Some(guild_id) = ctx.guild_id() {
        let gm_role_id;
        {
            let data = ctx.data().data.read().await;
            let guild = guild_id
                .to_guild_cached(ctx.serenity_context())
                .ok_or("Guild not in cache")?;

            gm_role_id = guild
                .role_by_name(&data.gm_role_name)
                .ok_or(format!("No such role `{}`", &data.gm_role_name))?
                .id
                .get();
        }
        if let Ok(member) = guild_id.member(ctx.serenity_context(), ctx.author().id).await {
            let has_role = member.roles.iter().any(|r| r.get() == gm_role_id);
            return Ok(has_role);
        }
    }

    Ok(false)
}
