use crate::ManagerKey;
use anyhow::Context as _;
use serenity::{
    client::Context,
    framework::standard::{macros::group, Args},
    model::channel::Message,
};
use sparky_macros::cmd;

use failures::*;

#[group]
#[prefix = "adm"]
#[owners_only]
#[commands(stop, clear, fail, crash)]
pub struct Admin;

#[cmd]
#[description = "Interrompt le bot après avoir correctement fermé la connexion."]
fn stop(ctx: &Context) {
    let data = ctx.data.read();
    let manager = data.get::<ManagerKey>().context("manager not in data")?;
    manager.lock().shutdown_all();
}

#[cmd]
#[description = "Supprime des anciens messages. Argument :\n\
+ nombre <= 100 alors exactement ce nombre de mesagges.\n\
+ nombre > 100 alors jusqu'au message ayant cet ID s'il est dans la limite.\n\
+ autre ou rien alors limite par défaut de Discord."]
fn clear(ctx: &Context, msg: &Message, mut args: Args) {
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
}

#[allow(unreachable_code)]
mod failures {
    use anyhow::bail;
    use sparky_macros::cmd;

    #[cmd]
    #[description = "Déclenche délibérément une erreur, à des fins de débogage."]
    pub fn fail() {
        bail!("failing on purpose")
    }

    #[cmd]
    #[description = "Provoque délibérément un crash du bot, à des fins de débogage."]
    pub fn crash() {
        panic!("panicking on purpose")
    }
}
