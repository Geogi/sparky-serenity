use crate::error::AVoid;
use crate::shadowrun::{shadowrun_reaction_add, shadowrun_reaction_remove};
use log::info;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Reaction;

pub struct Handler;
impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        fn inner(ctx: &Context, add_reaction: &Reaction) -> AVoid {
            shadowrun_reaction_add(ctx, add_reaction)?;
            Ok(())
        }
        if let Err(e) = inner(&ctx, &add_reaction) {
            info!("{:#}", e);
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        fn inner(ctx: &Context, removed_reaction: &Reaction) -> AVoid {
            shadowrun_reaction_remove(ctx, removed_reaction)?;
            Ok(())
        }
        if let Err(e) = inner(&ctx, &removed_reaction) {
            info!("{:#}", e);
        }
    }
}
