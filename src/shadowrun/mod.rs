use crate::shadowrun::{
    confirm::CONFIRM_COMMAND, plan::PLAN_COMMAND, remind::REMIND_COMMAND, roll::ROLL_COMMAND,
};
use anyhow::{anyhow, Context as _, Error};
use fehler::throws;
use serenity::{
    client::Context,
    framework::standard::macros::group,
    model::id::{RoleId, UserId},
    model::{channel::Reaction, guild::Role},
};

pub mod confirm;
pub mod plan;
pub mod remind;
pub mod roll;

match_env! {
pub const RUNNER: RoleId = match {
    prod => 293393770941251584,
    test => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[description = "Commandes liées au jeu de rôles papier Shadowrun."]
#[commands(plan, confirm, remind, roll)]
pub struct Shadowrun;

#[throws]
pub fn shadowrun_reaction(ctx: &Context, reaction: &Reaction) {
    plan::react(ctx, reaction).context("plan")?;
    confirm::react(ctx, reaction).context("confirm")?;
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
