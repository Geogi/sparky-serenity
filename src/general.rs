use serenity::{client::Context, framework::standard::macros::group, model::channel::Message};
use sparky_macros::cmd;

#[group]
#[commands(simple)]
pub struct General;

#[cmd]
#[description = "Liste les jours de la semaine en réaction."]
pub fn simple(ctx: &Context, msg: &Message) {
    msg.channel_id.send_message(ctx, |m| {
        m.content("Quel jour ?")
            .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
    })?;
}
