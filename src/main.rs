#[macro_use]
mod macros;

mod date;
mod discord;
mod edf;
mod error;
mod handler;
mod help;
mod shadowrun;
mod state;
mod utils;

use crate::edf::EDF_GROUP;
use crate::error::{log_cmd_err, wrap_cmd_err, AVoid};
use crate::handler::Handler;
use crate::help::MY_HELP;
use crate::shadowrun::roll::roll;
use crate::shadowrun::SHADOWRUN_GROUP;
use anyhow::anyhow;
use dotenv::dotenv;
use log::info;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::{Client, Context};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::Mutex as SerenityMutex;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use typemap::Key;

#[allow(clippy::unreadable_literal)]
const OWNER: UserId = UserId(190183362294579211);

match_guild! {
const PREFIX: &str = match {
    exylobby => "!",
    ytp => "?",
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

#[group]
#[prefix = "adm"]
#[owners_only]
#[commands(stop, clear)]
struct Admin;

#[command]
#[description = "Interrompt le bot après avoir correctement fermé la connexion."]
fn stop(ctx: &mut Context, _msg: &Message, mut _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let data = ctx.data.read();
        let manager = data
            .get::<ManagerKey>()
            .ok_or_else(|| anyhow!("manager not in data"))?;
        manager.lock().shutdown_all();
        Ok(())
    })
}

#[command]
#[description = "Supprime des anciens messages. Argument :\n\
+ nombre <= 100 alors exactement ce nombre de mesagges.\n\
+ nombre > 100 alors jusqu'au message ayant cet ID s'il est dans la limite.\n\
+ autre ou rien alors limite par défaut de Discord."]
fn clear(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let arg: Option<u64> = args.single().ok();
        let n_msg_before = |n| msg.channel_id.messages(ctx, |b| b.before(msg).limit(n));
        let messages = match arg {
            Some(n) if n <= 100 => n_msg_before(n)?,
            Some(n) if n > 100 => {
                let mut all = n_msg_before(100)?;
                loop {
                    let m = all.pop();
                    if let Some(m) = m {
                        if m.id == n {
                            all.push(m);
                            break;
                        }
                    } else {
                        break;
                    }
                }
                all
            }
            _ => n_msg_before(100)?,
        };
        msg.channel_id.delete_messages(ctx, messages)?;
        msg.delete(ctx)?;
        Ok(())
    })
}
