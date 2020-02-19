use dotenv::dotenv;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::env;

mod help;
mod shadowrun;

use help::MY_HELP;
use shadowrun::SHADOWRUN_GROUP;

const OWNER: u64 = 190183362294579211;

struct Handler;
impl EventHandler for Handler {}

fn main() {
    dotenv().ok();

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("!")
                    .owners(vec![UserId(OWNER)].into_iter().collect())
            })
            .group(&GENERAL_GROUP)
            .group(&SHADOWRUN_GROUP)
            .help(&MY_HELP),
    );

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[group]
#[commands(simple)]
struct General;

#[command]
fn simple(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx, |m| {
            m.content("Quel jour ?")
                .reactions(["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"].iter().cloned())
        })
        .ok();
    Ok(())
}
