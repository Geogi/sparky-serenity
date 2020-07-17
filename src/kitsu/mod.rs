mod greet;
pub mod parse;

use crate::kitsu::parse::BESTLOGS_COMMAND;
use anyhow::Error;
use fehler::throws;
use serenity::{
    client::Context,
    framework::standard::macros::group,
    model::{guild::Member, id::GuildId},
};

const GUILD_ID: GuildId = GuildId(587883211225563160);

#[group]
#[prefix = "ff"]
#[commands(bestlogs)]
pub struct Kitsu;

#[throws]
pub fn kitsu_add_member(ctx: &Context, guild_member: &(GuildId, Member)) {
    let (guild, member) = guild_member;
    if guild != &GUILD_ID {
        return;
    }
    greet::send_embed(ctx, member)?;
}
