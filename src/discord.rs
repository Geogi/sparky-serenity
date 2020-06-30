use crate::error::{ARes, AVoid};
use anyhow::Context as _;
use serenity::{
    client::Context,
    model::channel::{Message, Reaction},
    model::user::User,
};

pub fn pop_self(ctx: &Context, users: &mut Vec<User>) -> AVoid {
    let self_id = ctx.http.get_current_user()?.id;
    users.retain(|user| user.id != self_id);
    Ok(())
}

pub fn reaction_is_own(ctx: &Context, reaction: &Reaction) -> ARes<bool> {
    Ok(reaction.user_id == ctx.http.get_current_user()?.id)
}

pub fn delete_command_ifp(ctx: &Context, msg: &Message) -> AVoid {
    let guild_chan = msg
        .channel_id
        .to_channel(ctx)?
        .guild()
        .context("not a guild channel")?;
    let permissions = guild_chan
        .read()
        .permissions_for_user(ctx, ctx.http.get_current_user()?.id)?;
    if permissions.manage_messages() {
        msg.delete(ctx)?;
    }
    Ok(())
}
