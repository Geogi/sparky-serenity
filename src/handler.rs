use crate::error::{log_handler_err};
use crate::shadowrun::{shadowrun_reaction_add, shadowrun_reaction_remove};
use anyhow::{Context as _};
use serenity::client::{Context, EventHandler};
use serenity::model::{id::GuildId, channel::Reaction, guild::Member};

pub struct Handler;
impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        handle!("reaction_add" for ctx, add_reaction => {
            "shadowrun" => shadowrun_reaction_add,
        } in add_reaction.channel_id);
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        handle!("reaction_remove" for ctx, removed_reaction => {
            "shadowrun" => shadowrun_reaction_remove,
        } in removed_reaction.channel_id);
    }

    fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {
    }
}
