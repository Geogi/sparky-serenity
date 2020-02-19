#[macro_use]
mod macros;

mod date;
mod handler;
mod help;
mod shadowrun;
mod state;

use crate::handler::Handler;
use crate::help::MY_HELP;
use crate::shadowrun::SHADOWRUN_GROUP;
use dotenv::dotenv;
use serenity::client::{Client, Context};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::env;

const OWNER: u64 = 190183362294579211;

fn main() {
    dotenv().unwrap();

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
                .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
        })
        .unwrap();
    Ok(())
}
