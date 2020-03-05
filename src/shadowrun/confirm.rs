use crate::date::{
    fr_day_to_str, fr_month_to_str, fr_weekday_from_shorthand, fr_weekday_to_emote,
    fr_weekday_to_str,
};
use crate::error::{ARes, AVoid};
use crate::shadowrun::RUNNER;
use crate::state::get_state;
use crate::state::{encode, Embedded};
use crate::utils::{pop_self, reaction_is_own};
use anyhow::{anyhow, bail};
use chrono::{Date, Datelike, NaiveTime, TimeZone, Utc, Weekday};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::channel::{Message, Reaction};
use serenity::model::guild::Role;
use serenity::model::id::UserId;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;
use std::collections::HashMap;
use std::ops::Deref;

const DEFAULT_HOST: UserId = UserId(190183362294579211);
lazy_static! {
    static ref NOON: NaiveTime = NaiveTime::from_hms(0, 0, 0);
}

#[derive(Serialize, Deserialize)]
pub struct ShadowrunConfirm {
    date_timestamp: i64,
    participants_raw_ids: Vec<u64>,
}

#[command]
#[num_args(1)]
pub fn confirm(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let ctx = ctx.deref();
    let plan = get_last_plan(ctx, msg)?;
    let day = fr_weekday_from_shorthand(&args.single::<String>()?)
        .ok_or(anyhow!("cannot parse weekday"))?;
    let (participants, date) = read_participants_date(ctx, &plan, day)?;
    let data = ShadowrunConfirm {
        date_timestamp: date
            .and_time(*NOON)
            .ok_or("cannot build datetime")?
            .timestamp(),
        participants_raw_ids: participants.iter().map(|u| u.id.0).collect(),
    };
    let mut msg = msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| e.description("En préparation..."))
            .reactions(vec!["✅", "🚫", "🏠", "🚩", "🕣", "🕘", "🕤"])
    })?;
    refresh(ctx, &mut msg, data)?;
    Ok(())
}

pub fn confirm_react(ctx: &Context, reaction: &Reaction) -> AVoid {
    if reaction_is_own(ctx, reaction)? {
        return Ok(());
    }
    let mut msg = reaction.message(ctx)?;
    if let Some(Embedded::EShadowrunConfirm(data)) = get_state(ctx, &msg) {
        refresh(ctx, &mut msg, data)?;
    }
    Ok(())
}

fn refresh(ctx: &Context, msg: &mut Message, data: ShadowrunConfirm) -> AVoid {
    let runner: Role = RUNNER.to_role_cached(ctx).ok_or(anyhow!("no role"))?;
    let ShadowrunConfirm {
        date_timestamp,
        participants_raw_ids: participants_ids,
    } = data;
    let date = Utc.timestamp(date_timestamp, 0).date();
    let mut participants = HashMap::new();
    for user_id_raw in &participants_ids {
        participants.insert(
            UserId(*user_id_raw),
            ConfirmInfo {
                attendance: Attendance::Pending,
                hosting: Hosting::Unspecified,
                time: GameTime::Eight,
            },
        );
    }
    let data = encode(Embedded::EShadowrunConfirm(ShadowrunConfirm {
        date_timestamp,
        participants_raw_ids: participants_ids,
    }))?;
    let time = earliest(ctx, msg)?;
    let host = host_priority
    msg.edit(ctx, |m| {
        m.embed(|e| {
            e.title("Shadowrun – Confirmation")
                .colour(runner.colour)
                .description({
                    let mut mb = MessageBuilder::new();
                    for (user_id, info) in &participants {
                        match info.attendance {
                            Attendance::Confirmed => mb.push("✅"),
                            Attendance::Cancelled => mb.push("🚫"),
                            Attendance::Pending => mb.push("⌛"),
                        };
                        mb.mention(user_id).push(", ");
                    }
                    mb.push("\nLa prochaine séance aura lieu le ")
                        .push_bold(fr_weekday_to_str(date.weekday()))
                        .push(" ")
                        .push_bold(fr_day_to_str(date))
                        .push(" ")
                        .push_bold(fr_month_to_str(date))
                        .push(" à ")
                        .push_bold(time)
                        .push(" chez ")
                        .mention(&host_rules(&participants))
                        .push(".\nMerci de : ")
                        .push_bold("✅ confirmer 🚫 annuler")
                        .push(".\nAccueil : ")
                        .push_bold("🏠 possible 🚩 demandé")
                        .push(".\nDécaler l’horaire : ")
                        .push_bold("🕣 20h30 🕘 21h 🕤 21h30")
                        .push(".");
                    mb
                })
                .footer(|f| f.text(data))
        })
    })?;
    Ok(())
}

fn earliest(ctx: &Context, msg: &Message) -> ARes<&'static str> {
    for (emote, string) in &[("🕤", "21h30"), ("🕘", "21h"), ("🕣", "20h30")] {
        let mut reactions =
            msg.reaction_users(ctx, Unicode(emote.to_owned().to_owned()), None, None)?;
        pop_self(ctx, &mut reactions)?;
        if !reactions.is_empty() {
            return Ok(string);
        }
    }
    Ok("20h")
}

struct ConfirmInfo {
    attendance: Attendance,
    hosting: Hosting,
    time: GameTime,
}

enum Attendance {
    Confirmed,
    Cancelled,
    Pending,
}

enum Hosting {
    Priority,
    Granted,
    Unspecified,
}

enum GameTime {
    Eight,
    EightThirty,
    Nine,
    NineThirty,
}

fn get_last_plan(ctx: &Context, base: &Message) -> ARes<Message> {
    for msg in base.channel_id.messages(ctx, |r| r.before(base.id))? {
        if let Some(Embedded::EShadowrunPlan(_)) = get_state(ctx, &msg) {
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
