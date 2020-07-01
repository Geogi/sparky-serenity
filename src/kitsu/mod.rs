use anyhow::Error;
use fehler::throws;
use serenity::{
    client::Context,
    model::{guild::Member, id::GuildId}, framework::standard::macros::group,
};
use crate::kitsu::parse::PARSE_COMMAND;

mod greet;
pub mod parse;

const GUILD_ID: GuildId = GuildId(587883211225563160);

#[group]
#[prefix = "ff"]
#[commands(parse)]
pub struct Kitsu;

#[throws]
pub fn kitsu_add_member(ctx: &Context, guild_member: &(GuildId, Member)) {
    let (guild, member) = guild_member;
    if guild != &GUILD_ID {
        return;
    }
    greet::send_embed(ctx, member)?;
}
