
use serenity::{client::Context, framework::standard::{Args, macros::{command, group}, CommandResult}, model::channel::Message};
use crate::{ManagerKey, error::wrap_cmd_err};
use anyhow::bail;
use anyhow::Context as _;

#[group]
#[prefix = "adm"]
#[owners_only]
#[commands(stop, clear, fail, crash)]
pub struct Admin;

#[command]
#[description = "Interrompt le bot après avoir correctement fermé la connexion."]
fn stop(ctx: &mut Context, _msg: &Message, mut _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let data = ctx.data.read();
        let manager = data
            .get::<ManagerKey>()
            .context("manager not in data")?;
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

#[command]
#[description = "Déclenche délibérément une erreur, à des fins de débogage."]
fn fail() -> CommandResult {
    wrap_cmd_err(|| 
        bail!("failing on purpose")
    )
}

#[command]
#[description = "Provoque délibérément un crash du bot, à des fins de débogage."]
fn crash() -> CommandResult {
    panic!("panicking on purpose")
}
