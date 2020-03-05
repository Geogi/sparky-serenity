#[macro_use]
mod macros;

mod date;
mod error;
mod handler;
mod help;
mod shadowrun;
mod state;
mod utils;

use crate::error::{log_err, AVoid};
use crate::handler::Handler;
use crate::help::MY_HELP;
use crate::shadowrun::SHADOWRUN_GROUP;
use anyhow::Context as _;
use dotenv::dotenv;
use serenity::client::{Client, Context};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::env;

const OWNER: u64 = 190183362294579211;

fn main() -> AVoid {
    env_logger::init();
    dotenv()?;

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("!").owners({
                    let mut owners = HashSet::new();
                    owners.insert(UserId(OWNER));
                    owners
                })
            })
            .after(log_err)
            .group(&GENERAL_GROUP)
            .group(&SHADOWRUN_GROUP)
            .help(&MY_HELP),
    );

    client
        .start()
        .context("An error occurred while running the client")
}

#[group]
#[commands(simple)]
struct General;

#[command]
fn simple(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx, |m| {
        m.content("Quel jour ?")
            .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
    })?;
    Ok(())
}
