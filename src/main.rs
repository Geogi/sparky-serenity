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
use anyhow::anyhow;
use dotenv::dotenv;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::{Client, Context};
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::{Mutex, TypeMapKey};
use std::collections::HashSet;
use std::env;
use std::ops::Deref;
use std::sync::Arc;

const OWNER: u64 = 190183362294579211;

struct ManagerKey;
impl TypeMapKey for ManagerKey {
    type Value = Arc<Mutex<ShardManager>>;
}

fn main() -> AVoid {
    env_logger::init();
    dotenv()?;

    let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
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
            .group(&ADMIN_GROUP)
            .group(&SHADOWRUN_GROUP)
            .help(&MY_HELP),
    );

    client
        .data
        .write()
        .insert::<ManagerKey>(client.shard_manager.clone());

    client.start()?;

    Ok(())
}

#[group]
#[commands(simple)]
struct General;

#[command]
fn simple(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ctx = ctx.deref();
    msg.channel_id.send_message(ctx, |m| {
        m.content("Quel jour ?")
            .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
    })?;
    Ok(())
}

#[group]
#[prefix = "adm"]
#[owners_only]
#[commands(stop)]
struct Admin;

#[command]
fn stop(ctx: &mut Context) -> CommandResult {
    let ctx = ctx.deref();
    let data = ctx.data.read();
    let manager = data
        .get::<ManagerKey>()
        .ok_or(anyhow!("manager not in data"))?;
    manager.lock().shutdown_all();
    Ok(())
}
