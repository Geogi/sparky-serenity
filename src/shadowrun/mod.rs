pub mod confirm;
pub mod plan;

use crate::error::AVoid;
use crate::shadowrun::confirm::CONFIRM_COMMAND;
use crate::shadowrun::plan::{plan_edit, PLAN_COMMAND};
use serenity::client::Context;
use serenity::framework::standard::macros::group;
use serenity::model::channel::Reaction;
use serenity::model::id::RoleId;

match_id! {
pub const RUNNER: RoleId = match {
    exylobby => 293393770941251584,
    ytp => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[commands(plan, confirm)]
pub struct Shadowrun;

pub fn shadowrun_reaction_add(ctx: &Context, add_reaction: &Reaction) -> AVoid {
    plan_edit(ctx, add_reaction)?;
    Ok(())
}

pub fn shadowrun_reaction_remove(ctx: &Context, removed_reaction: &Reaction) -> AVoid {
    plan_edit(ctx, removed_reaction)?;
    Ok(())
}
