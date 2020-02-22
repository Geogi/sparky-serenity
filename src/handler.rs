use crate::shadowrun::{shadowrun_reaction_add, shadowrun_reaction_remove};
use serenity::client::EventHandler;
use serenity::model::channel::Reaction;
use serenity::prelude::Context;

pub struct Handler;
impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        shadowrun_reaction_add(ctx, add_reaction);
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        shadowrun_reaction_remove(ctx, removed_reaction);
    }
}
