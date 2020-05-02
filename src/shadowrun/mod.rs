pub mod confirm;
pub mod plan;
pub mod remind;
pub mod roll;

use crate::error::{ARes, AVoid};
use crate::shadowrun::confirm::{confirm_react, CONFIRM_COMMAND};
use crate::shadowrun::plan::{plan_react, PLAN_COMMAND};
use crate::shadowrun::remind::REMIND_COMMAND;
use crate::shadowrun::roll::ROLL_COMMAND;
use anyhow::{anyhow, Context as _};
use serenity::client::Context;
use serenity::framework::standard::macros::group;
use serenity::model::channel::Reaction;
use serenity::model::guild::Role;
use serenity::model::id::{RoleId, UserId};

match_guild! {
pub const RUNNER: RoleId = match {
    exylobby => 706172165510267010,
    ytp => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[commands(plan, confirm, remind, roll)]
pub struct Shadowrun;

pub fn shadowrun_reaction_add(ctx: &Context, add_reaction: &Reaction) -> AVoid {
    plan_react(ctx, add_reaction).context("plan")?;
    confirm_react(ctx, add_reaction).context("confirm")?;
    Ok(())
}

pub fn shadowrun_reaction_remove(ctx: &Context, removed_reaction: &Reaction) -> AVoid {
    plan_react(ctx, removed_reaction).context("plan")?;
    confirm_react(ctx, removed_reaction).context("confirm")?;
    Ok(())
}

pub fn runners(ctx: &Context) -> ARes<Vec<UserId>> {
    let runner: Role = RUNNER
        .to_role_cached(ctx)
        .ok_or_else(|| anyhow!("no role"))?;
    let guild = runner
        .find_guild(ctx)?
        .to_guild_cached(ctx)
        .ok_or_else(|| anyhow!("cannot read guild"))?;
    let guild = guild.read();
    let runners = guild.members.iter().filter_map(|(id, member)| {
        if member.roles.contains(&RUNNER) {
            Some(id)
        } else {
            None
        }
    });
    Ok(runners.cloned().collect())
}
