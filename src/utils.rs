use crate::error::{AVoid, ARes};
use serenity::client::Context;
use serenity::model::user::User;
use serenity::model::channel::Reaction;

pub fn pop_self(ctx: &Context, users: &mut Vec<User>) -> AVoid {
    let self_id = ctx.http.get_current_user()?.id;
    users.retain(|user| user.id != self_id);
    Ok(())
}

pub fn reaction_is_own(ctx: &Context, reaction: &Reaction) -> ARes<bool> {
    Ok(reaction.user_id == ctx.http.get_current_user()?.id)
}
