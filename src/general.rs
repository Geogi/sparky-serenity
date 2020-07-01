use crate::error::wrap_cmd_err;
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
};

#[group]
#[commands(simple)]
pub struct General;

#[command]
#[description = "Liste les jours de la semaine en réaction."]
pub fn simple(ctx: &mut Context, msg: &Message, mut _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        msg.channel_id.send_message(ctx, |m| {
            m.content("Quel jour ?")
                .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
        })?;
        Ok(())
    })
}
