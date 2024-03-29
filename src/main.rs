#[macro_use]
mod macros;

mod admin;
mod date;
mod discord;
mod edf;
mod error;
mod general;
mod handler;
mod help;
mod shadowrun;
mod state;
mod string;
mod utils;
mod vote;

use crate::{
    admin::ADMIN_GROUP, edf::EDF_GROUP, error::log_cmd_err, general::GENERAL_GROUP,
    handler::Handler, help::MY_HELP, shadowrun::SHADOWRUN_GROUP,
};
use anyhow::Error;
use dotenv::dotenv;
use fehler::throws;
use log::info;
use serenity::{
    client::{bridge::gateway::ShardManager, Client},
    framework::standard::StandardFramework,
    model::id::UserId,
    prelude::Mutex as SerenityMutex,
};
use sparky_macros::shortcuts;
use std::{collections::HashSet, env, sync::Arc};

#[allow(clippy::unreadable_literal)]
const OWNER: UserId = UserId(190183362294579211);

match_env! {
const PREFIX: &str = match {
    prod => "!",
    test => "?",
}}

struct ManagerKey;
impl typemap::Key for ManagerKey {
    type Value = Arc<SerenityMutex<ShardManager>>;
}

#[throws]
fn main() {
    dotenv()?;
    env_logger::init();

    let mut client = Client::new(&env::var("DISCORD_TOKEN")?, Handler)?;
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix(PREFIX).owners({
                    let mut owners = HashSet::new();
                    owners.insert(OWNER);
                    owners
                })
            })
            .after(log_cmd_err)
            .group(&SHORTCUT_GROUP)
            .group(&ADMIN_GROUP)
            .group(&EDF_GROUP)
            .group(&GENERAL_GROUP)
            .group(&SHADOWRUN_GROUP)
            .help(&MY_HELP),
    );

    let interrupt_manager = client.shard_manager.clone();
    ctrlc::set_handler(move || {
        info!("received termination signal");
        interrupt_manager.lock().shutdown_all();
    })?;

    client
        .data
        .write()
        .insert::<ManagerKey>(client.shard_manager.clone());

    client.start()?;
}

shortcuts! {
    match () {
        r => shadowrun::roll::roll,
        simple => general::simple,
    }
}
