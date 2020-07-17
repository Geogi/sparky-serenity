use serenity::{client::Context, framework::standard::{CommandResult, macros::command, Args}, model::channel::Message};
use crate::error::wrap_cmd_err;

#[command]
pub fn start(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        Ok(())
    })
}
