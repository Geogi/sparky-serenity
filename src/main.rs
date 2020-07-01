use crate::{
    admin::ADMIN_GROUP,
    edf::EDF_GROUP,
    error::{log_cmd_err, wrap_cmd_err, AVoid},
    handler::Handler,
    help::MY_HELP,
    shadowrun::roll::roll,
    shadowrun::SHADOWRUN_GROUP,
};
use dotenv::dotenv;
use log::info;
use serenity::{
    client::{bridge::gateway::ShardManager, Client, Context},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::channel::Message,
    model::id::UserId,
    prelude::Mutex as SerenityMutex,
};
use std::{collections::HashSet, env, sync::Arc};
use typemap::Key;

#[macro_use]
mod macros;

mod admin;
mod date;
mod discord;
mod edf;
mod error;
mod handler;
mod help;
mod kitsu;
mod shadowrun;
mod state;
mod utils;

#[allow(clippy::unreadable_literal)]
const OWNER: UserId = UserId(190183362294579211);

match_guild! {
const PREFIX: &str = match {
    prod => "!",
    test => "?",
}}

struct ManagerKey;
impl Key for ManagerKey {
    type Value = Arc<SerenityMutex<ShardManager>>;
}

fn main() -> AVoid {
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
            .group(&GENERAL_GROUP)
            .group(&ADMIN_GROUP)
            .group(&EDF_GROUP)
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

    Ok(())
}

#[group]
#[commands(simple, roll_shortcut)]
struct General;

#[command]
#[description = "Raccourci pour `sr roll`"]
#[help_available(false)]
#[aliases(r)]
fn roll_shortcut(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    roll(ctx, msg, args)
}

#[command]
#[description = "Liste les jours de la semaine en réaction."]
fn simple(ctx: &mut Context, msg: &Message, mut _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        msg.channel_id.send_message(ctx, |m| {
            m.content("Quel jour ?")
                .reactions(vec!["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"])
        })?;
        Ok(())
    })
}
