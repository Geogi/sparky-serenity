use crate::error::{log_handler_err, AVoid};
use crate::shadowrun::{shadowrun_reaction_add, shadowrun_reaction_remove};
use anyhow::Context as _;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Reaction;

pub struct Handler;
impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        fn inner(ctx: &Context, add_reaction: &Reaction) -> AVoid {
            shadowrun_reaction_add(ctx, add_reaction).context("shadowrun")?;
            Ok(())
        }
        log_handler_err(
            &ctx,
            add_reaction.channel_id,
            inner(&ctx, &add_reaction).context("`reaction_add`"),
        );
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        fn inner(ctx: &Context, removed_reaction: &Reaction) -> AVoid {
            shadowrun_reaction_remove(ctx, removed_reaction).context("shadowrun")?;
            Ok(())
        }
        log_handler_err(
            &ctx,
            removed_reaction.channel_id,
            inner(&ctx, &removed_reaction).context("`reaction_remove`"),
        );
    }
}
