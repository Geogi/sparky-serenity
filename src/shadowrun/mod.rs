mod confirm;
pub mod plan;

use crate::shadowrun::confirm::CONFIRM_COMMAND;
use crate::shadowrun::plan::{plan_edit, PLAN_COMMAND};
use serenity::client::Context;
use serenity::framework::standard::macros::group;
use serenity::model::channel::Reaction;

#[group]
#[prefix = "sr"]
#[commands(plan, confirm)]
pub struct Shadowrun;

pub fn shadowrun_reaction_add(ctx: Context, add_reaction: Reaction) {
    plan_edit(ctx, add_reaction);
}

pub fn shadowrun_reaction_remove(ctx: Context, removed_reaction: Reaction) {
    plan_edit(ctx, removed_reaction);
}
