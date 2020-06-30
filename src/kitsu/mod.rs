use anyhow::Error;
use fehler::throws;
use serenity::{
    client::Context,
    model::{guild::Member, id::GuildId},
};

mod greet;

#[throws]
pub fn kitsu_add_member(_ctx: &Context, _guild_member: &(GuildId, Member)) {
    
}
