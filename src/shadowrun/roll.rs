use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[command]
pub fn roll(_ctx: &mut Context, _msg: &Message, _args: Args) -> CommandResult {
    Ok(())
}
