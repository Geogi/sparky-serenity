use anyhow::Error;
use fehler::throws;
use serenity::{
    client::Context,
    model::{guild::Member, id::GuildId},
};

mod greet;

const GUILD_ID: GuildId = GuildId(587883211225563160);

#[throws]
pub fn kitsu_add_member(ctx: &Context, guild_member: &(GuildId, Member)) {
    let (guild, member) = guild_member;
    if guild != &GUILD_ID {
        return;
    }
    greet::send_embed(ctx, member)?;
}
