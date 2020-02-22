use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

#[command]
pub fn confirm(_ctx: &mut Context, _msg: &Message) -> CommandResult {
    Ok(())
}
