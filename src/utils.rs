use crate::error::AVoid;
use serenity::client::Context;
use serenity::model::user::User;

pub fn pop_self(ctx: &Context, users: &mut Vec<User>) -> AVoid {
    let self_id = ctx.http.get_current_user()?.id;
    users.retain(|user| user.id != self_id);
    Ok(())
}
