pub mod confirm;
pub mod plan;
pub mod remind;
pub mod roll;

use crate::shadowrun::confirm::{confirm_react, CONFIRM_COMMAND};
use crate::shadowrun::plan::{plan_react, PLAN_COMMAND};
use crate::shadowrun::remind::REMIND_COMMAND;
use crate::shadowrun::roll::ROLL_COMMAND;
use anyhow::{anyhow, Error, Context as _};
use serenity::client::Context;
use serenity::framework::standard::macros::group;
use serenity::model::channel::Reaction;
use serenity::model::guild::Role;
use serenity::model::id::{RoleId, UserId};
use fehler::throws;

match_guild! {
pub const RUNNER: RoleId = match {
    exylobby => 293393770941251584,
    ytp => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[commands(plan, confirm, remind, roll)]
pub struct Shadowrun;

#[throws]
pub fn shadowrun_reaction(ctx: &Context, reaction: &Reaction) {
    plan_react(ctx, reaction).context("plan")?;
    confirm_react(ctx, reaction).context("confirm")?;
}

#[throws]
pub fn runners(ctx: &Context) -> Vec<UserId> {
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
    runners.cloned().collect()
}
