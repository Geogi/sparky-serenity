use crate::date::{
    fr_day_to_str, fr_month_to_str, fr_weekday_from_shorthand, fr_weekday_to_emote,
    fr_weekday_to_str,
};
use crate::error::ARes;
use crate::shadowrun::plan::is_sr_plan;
use crate::shadowrun::RUNNER;
use crate::state::{encode, Embedded};
use crate::utils::pop_self;
use anyhow::{anyhow, bail};
use chrono::{Date, Datelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::guild::Role;
use serenity::model::id::UserId;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;
use std::ops::Deref;

const DEFAULT_HOST: UserId = UserId(190183362294579211);

#[command]
#[num_args(1)]
pub fn confirm(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let ctx = ctx.deref();
    let runner: Role = RUNNER.to_role_cached(ctx).ok_or(anyhow!("no role"))?;
    let day = fr_weekday_from_shorthand(&args.single::<String>()?)
        .ok_or(anyhow!("cannot parse weekday"))?;
    let plan = get_last_plan(ctx, msg)?;
    let (participants, date) = read_participants_date(ctx, &plan, day)?;
    let data = encode(Embedded::EShadowrunConfirm(ShadowrunConfirm))?;
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Shadowrun – Confirmation")
                .colour(runner.colour)
                .description({
                    let mut mb = MessageBuilder::new();
                    for user in participants {
                        mb.mention(&user);
                        mb.push(", ");
                    }
                    mb.push("la prochaine séance aura lieu le ")
                        .push(fr_weekday_to_str(date.weekday()))
                        .push(" ")
                        .push(fr_day_to_str(date))
                        .push(" ")
                        .push(fr_month_to_str(date))
                        .push(" à 20h chez ")
                        .mention(&DEFAULT_HOST)
                        .push(".\nMerci de : ✅ confirmer 🚫 annuler.\nAccueil : 🏠 possible 🏚 impossible 🚩 demandé ")
                        .push_italic("(cf. règles)")
                        .push(".\nDécaler l’horaire : 🕣 20h30 🕘 21h 🕤 21h30.");
                    mb
                })
                .footer(|f| f.text(data))
        })
            .reactions(vec!["✅", "🚫", "🏠", "🏚", "🚩", "🕣", "🕘", "🕤"])
    })?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ShadowrunConfirm;

fn get_last_plan(ctx: &Context, base: &Message) -> ARes<Message> {
    for msg in base.channel_id.messages(ctx, |r| r.before(base.id))? {
        if is_sr_plan(ctx, &msg) {
            return Ok(msg);
        }
    }
    bail!("could not find plan message")
}

fn read_participants_date(
    ctx: &Context,
    plan: &Message,
    day: Weekday,
) -> ARes<(Vec<User>, Date<Utc>)> {
    let mut participants = plan.reaction_users(
        ctx,
        Unicode(fr_weekday_to_emote(day).to_owned()),
        None,
        None,
    )?;
    pop_self(ctx, &mut participants)?;
    let mut date = plan.timestamp.with_timezone(&Utc).date();
    while date.weekday() != day {
        date = date.succ();
    }
    Ok((participants, date))
}
