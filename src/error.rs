use anyhow::anyhow;
use log::info;
use serenity::client::Context;
use serenity::framework::standard::{CommandError, CommandResult};
use serenity::model::channel::Message;

pub type AVoid = anyhow::Result<()>;
pub type ARes<T> = anyhow::Result<T>;

pub fn log_err(_ctx: &mut Context, _msg: &Message, cmd: &str, res: CommandResult) {
    if let Err(CommandError(err)) = res {
        info!(
            "{:#}",
            anyhow!("{}", err).context(format!("command {} failed", cmd))
        );
    }
}
