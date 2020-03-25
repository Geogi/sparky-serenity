use crate::date::{
    fr_day_to_str, fr_month_to_str, fr_weekday_from_shorthand, fr_weekday_to_emote,
    fr_weekday_to_str,
};
use crate::discord::{pop_self, reaction_is_own};
use crate::error::{wrap_cmd_err, ARes, AVoid};
use crate::shadowrun::RUNNER;
use crate::state::extract;
use crate::state::{encode, Embedded};
use crate::utils::{find_message, MapExt};
use anyhow::{anyhow, bail, Context as _};
use boolinator::Boolinator;
use chrono::{Date, Datelike, TimeZone, Utc, Weekday};
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

#[allow(clippy::unreadable_literal)]
const DEFAULT_HOST: UserId = UserId(190183362294579211);
#[allow(clippy::unreadable_literal)]
const HOST_PRIORITY: &[UserId] = &[
    UserId(285875416860983306),
    UserId(172786235171930113),
    UserId(362692048039444492),
    UserId(136938893432848385),
];

#[derive(Serialize, Deserialize, Clone)]
pub struct ShadowrunConfirm {
    pub date_timestamp: i64,
    pub participants_raw_ids: Vec<u64>,
}

#[command]
#[num_args(1)]
pub fn confirm(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let plan = last_plan(ctx, msg)?;
        let day = fr_weekday_from_shorthand(&args.single::<String>()?)
            .ok_or_else(|| anyhow!("cannot parse weekday"))?;
        let (participants, date) = read_participants_date(ctx, &plan, day)?;
        let data = ShadowrunConfirm {
            date_timestamp: date.and_hms(12, 0, 0).timestamp(),
            participants_raw_ids: participants.iter().map(|u| u.id.0).collect(),
        };
        let mut msg = msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| e.description("En préparation..."))
                .reactions(vec!["✅", "🚫", "🏠", "🚩", "🕣", "🕘", "🕤"])
        })?;
        refresh(ctx, &mut msg, data).context("refresh embed")?;
        Ok(())
    })
}

pub fn confirm_react(ctx: &Context, reaction: &Reaction) -> AVoid {
    if reaction_is_own(ctx, reaction)? {
        return Ok(());
    }
    let mut msg = reaction.message(ctx)?;
    if let Some(Embedded::EShadowrunConfirm(data)) = extract(ctx, &msg) {
        refresh(ctx, &mut msg, data).context("embed refresh")?;
    }
    Ok(())
}

fn refresh(ctx: &Context, msg: &mut Message, data: ShadowrunConfirm) -> AVoid {
    let runner: Role = RUNNER
        .to_role_cached(ctx)
        .ok_or_else(|| anyhow!("no role"))?;
    let ShadowrunConfirm {
        date_timestamp,
        participants_raw_ids: participants_ids,
    } = data.clone();
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
    let rus = |em: &str| -> ARes<Vec<User>> {
        let mut users = msg.reaction_users(ctx, Unicode(em.to_owned()), None, None)?;
        pop_self(ctx, &mut users)?;
        Ok(users)
    };
    for confirming in rus("✅")? {
        participants.insert(
            confirming.id,
            ConfirmInfo {
                attendance: Attendance::Confirmed,
                hosting: Hosting::Unspecified,
                time: GameTime::Eight,
            },
        );
    }
    for cancelling in rus("🚫")? {
        participants.modify(cancelling.id, |ConfirmInfo { hosting, time, .. }| {
            ConfirmInfo {
                attendance: Attendance::Cancelled,
                hosting,
                time,
            }
        });
    }
    for granting in rus("🏠")? {
        participants.modify(
            granting.id,
            |ConfirmInfo {
                 attendance, time, ..
             }| ConfirmInfo {
                attendance,
                hosting: Hosting::Granted,
                time,
            },
        );
    }
    for demanding in rus("🚩")? {
        participants.modify(
            demanding.id,
            |ConfirmInfo {
                 attendance, time, ..
             }| ConfirmInfo {
                attendance,
                hosting: Hosting::Demanded,
                time,
            },
        );
    }
    for demanding in rus("🕣")? {
        participants.modify(
            demanding.id,
            |ConfirmInfo {
                 attendance,
                 hosting,
                 ..
             }| ConfirmInfo {
                attendance,
                hosting,
                time: GameTime::EightThirty,
            },
        );
    }
    for demanding in rus("🕘")? {
        participants.modify(
            demanding.id,
            |ConfirmInfo {
                 attendance,
                 hosting,
                 ..
             }| ConfirmInfo {
                attendance,
                hosting,
                time: GameTime::Nine,
            },
        );
    }
    for demanding in rus("🕤")? {
        participants.modify(
            demanding.id,
            |ConfirmInfo {
                 attendance,
                 hosting,
                 ..
             }| ConfirmInfo {
                attendance,
                hosting,
                time: GameTime::NineThirty,
            },
        );
    }
    let data = encode(Embedded::EShadowrunConfirm(data))?;
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
                        .push_bold(earliest(&participants))
                        .push(" chez ")
                        .mention(host_priority(&participants))
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

fn earliest(participants: &HashMap<UserId, ConfirmInfo>) -> &'static str {
    match participants
        .iter()
        .filter_map(|(_, info)| (info.attendance == Attendance::Confirmed).as_some(info.time))
        .max()
    {
        Some(GameTime::NineThirty) => "21h30",
        Some(GameTime::Nine) => "21h",
        Some(GameTime::EightThirty) => "20h30",
        _ => "20h",
    }
}

fn host_priority(participants: &HashMap<UserId, ConfirmInfo>) -> &UserId {
    for offer in &[Hosting::Demanded, Hosting::Granted] {
        let hosts = participants.iter().filter_map(|(id, info)| {
            (info.attendance == Attendance::Confirmed && &info.hosting == offer).as_some(id)
        });
        for priority_host in HOST_PRIORITY {
            if let Some(host) = hosts.clone().find(|&h| h == priority_host) {
                return host;
            }
        }
    }
    &DEFAULT_HOST
}

struct ConfirmInfo {
    attendance: Attendance,
    hosting: Hosting,
    time: GameTime,
}

#[derive(PartialEq)]
enum Attendance {
    Confirmed,
    Cancelled,
    Pending,
}

#[derive(PartialEq)]
enum Hosting {
    Unspecified,
    Granted,
    Demanded,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
enum GameTime {
    Eight,
    EightThirty,
    Nine,
    NineThirty,
}

fn last_plan(ctx: &Context, base: &Message) -> ARes<Message> {
    if let Ok(msg) = find_message(ctx, base, |d| matches!(d, Embedded::EShadowrunPlan(_))) {
        Ok(msg)
    } else {
        base.reply(ctx, "je n’ai pas trouvé le dernier planning.")?;
        bail!("could not find plan message")
    }
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
