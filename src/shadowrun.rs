use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

#[group]
#[prefix = "sr"]
#[commands(plan)]
pub struct Shadowrun;

#[command]
fn plan(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx, |m| {
            m.content("a").embed(|e| e.title("au").description("iue"))
        })
        .ok();
    Ok(())
}
