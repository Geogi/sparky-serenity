use crate::OWNER;
use anyhow::anyhow;
use chrono::{DateTime, Duration, Utc};
use log::info;
use serenity::{
    client::Context,
    framework::standard::{CommandError, CommandResult},
    model::channel::Message,
    model::id::ChannelId,
    utils::MessageBuilder,
};
use typemap::Key;

const HANDLER_REPORT_INTERVAL_SECONDS: i64 = 20;
const ERROR_CHAN: ChannelId = ChannelId(727584970520002651);

pub type AVoid = anyhow::Result<()>;
pub type ARes<T> = anyhow::Result<T>;

pub struct LastHandlerReportKey;
impl Key for LastHandlerReportKey {
    type Value = DateTime<Utc>;
}

pub fn wrap_cmd_err(f: impl FnOnce() -> AVoid) -> CommandResult {
    f().map_err(|e| CommandError(format!("{:#}", e)))
}

pub fn log_cmd_err(ctx: &mut Context, msg: &Message, cmd: &str, res: CommandResult) {
    let ctx = &*ctx;
    if let Err(CommandError(err)) = res {
        let chained = anyhow!("{}", err).context(format!("Échec de la commande `{}`", cmd));
        info!("{:#}", chained);
        // notifying through Discord is best efforts: result is ignored
        let _ = msg.reply(ctx, "La commande a échoué, nous travaillons dessus !");
        let _ = ERROR_CHAN.send_message(ctx, |m| m.content(format!("{:#}", chained)));
    }
}

pub fn log_handler_err(ctx: &Context, res: AVoid) {
    if let Err(err) = res {
        let chained = err.context("Erreur lors du traitement d’un événement");
        info!("{:#}", chained);
        let do_report = {
            let data = ctx.data.read();
            let last_handler_report = data.get::<LastHandlerReportKey>();
            last_handler_report
                .map(|v| Utc::now() - *v > Duration::seconds(HANDLER_REPORT_INTERVAL_SECONDS))
                .unwrap_or(true)
        };
        if do_report {
            // notifying through Discord is best efforts: result is ignored
            let _ = ERROR_CHAN.send_message(ctx, |m| {
                m.content(
                    MessageBuilder::new()
                        .mention(&OWNER)
                        .push(format!(": {:#}\n", chained))
                        .push("Les erreurs similaires seront ignorées durant ")
                        .push(HANDLER_REPORT_INTERVAL_SECONDS)
                        .push(" secondes mais restent accessibles dans le journal système."),
                )
            });
            let mut data = ctx.data.write();
            data.insert::<LastHandlerReportKey>(Utc::now());
        }
    }
}
