use crate::{error::log_handler_err, shadowrun::shadowrun_reaction};
use anyhow::Context as _;
use serenity::{
    client::{Context, EventHandler},
    model::channel::Reaction,
};

pub struct Handler;
impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        handle!("reaction_add" for ctx, add_reaction => {
            "shadowrun" => shadowrun_reaction,
        });
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        handle!("reaction_remove" for ctx, removed_reaction => {
            "shadowrun" => shadowrun_reaction,
        });
    }
}
