use crate::date::{
    fr_day_to_str, fr_month_to_str, fr_weekday_from_shorthand, fr_weekday_to_emote,
    fr_weekday_to_str, parse_time_emote_like, time_emote, TZ_DEFAULT, hm24_format,
};
use crate::discord::{pop_self, reaction_is_own};
use crate::error::{wrap_cmd_err, ARes, AVoid};
use crate::help::{clap_help, clap_settings};
use crate::shadowrun::RUNNER;
use crate::state::{encode, Embedded};
use crate::state::{extract, find_by_state};
use crate::utils::{clap_name, MapExt};
use anyhow::Error;
use anyhow::{anyhow, bail, Context as _};
use boolinator::Boolinator;
use chrono::Duration;
use chrono::{Date, Datelike, NaiveTime, TimeZone, Timelike, Weekday};
use clap::{App, Arg};
use fehler::throws;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::channel::{Message, Reaction};
use serenity::model::guild::Role;
use serenity::model::id::UserId;
use serenity::model::misc::Mentionable;
use serenity::model::user::User;
use serenity::utils::MessageBuilder;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use Attendance::{Cancelled, Confirmed, Pending};
use Hosting::{Demanded, Granted, Unspecified};

#[allow(clippy::unreadable_literal)]
const DEFAULT_HOST: UserId = UserId(190183362294579211);
#[allow(clippy::unreadable_literal)]
const HOST_PRIORITY: &[UserId] = &[
    UserId(285875416860983306),
    UserId(172786235171930113),
    UserId(362692048039444492),
    UserId(136938893432848385),
];

type HourHalf = (u8, bool);

#[derive(Serialize, Deserialize, Clone)]
pub struct ShadowrunConfirm {
    pub date_timestamp: i64,
    pub participants_raw_ids: Vec<u64>,
    pub online: bool,
    pub time: HourHalf,
    pub alt_times: Vec<HourHalf>,
}

#[command]
#[description = "Lit le dernier planning et crée un message de confirmation pour un jour donné.\n\
***ILC :** appelez avec `--help` pour l’utilisation.*"]
pub fn confirm(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let app = App::new(clap_name("sr confirm"))
            .about(
                "Lit le dernier planning et crée un message de confirmation pour un jour \
                donné.",
            )
            .arg(
                Arg::with_name("JOUR")
                    .help("Lettre du jour de la semaine choisi (LAEJVSD).")
                    .possible_values(&[
                        "l", "a", "e", "j", "v", "s", "d", "L", "A", "E", "J", "V", "S", "D",
                    ]),
            )
            .arg(
                Arg::with_name("online")
                    .short("o")
                    .help("Cette séance sera en ligne."),
            )
            .arg(
                Arg::with_name("time")
                    .short("t")
                    .takes_value(true)
                    .help("Horaire proposé par défaut.")
                    .default_value("20"),
            )
            .arg(
                Arg::with_name("alt-time")
                    .short("T")
                    .takes_value(true)
                    .multiple(true)
                    .help(
                        "Horaires alternatifs par précédence croissante. \
                    Par défaut, 3 demi-heures suivantes.",
                    ),
            );
        let app = clap_settings(app);
        let args = match clap_help(ctx, msg, args, app)? {
            Some(args) => args,
            None => return Ok(()),
        };
        let plan = last_plan(ctx, msg)?;
        let day = fr_weekday_from_shorthand(
            args.value_of("JOUR")
                .ok_or_else(|| anyhow!("unreachable: unspecified day"))?,
        )?;
        let online = args.is_present("online");
        let (participants, date) = read_participants_date(ctx, &plan, day, online, TZ_DEFAULT)?;
        let time = parse_time_emote_like(
            args.value_of("time")
                .context("unreachable: default value")?,
        )?;
        let mut reactions = vec!["✅", "🚫"];
        if !online {
            reactions.append(&mut vec!["🏠", "🚩"]);
        }
        let mut alt_times = vec![];
        if let Some(it) = args.values_of("alt-time") {
            for alt_time_str in it {
                let alt_time = parse_time_emote_like(alt_time_str)?;
                let emote = time_emote(alt_time)?;
                if alt_time == time {
                    msg.reply(ctx, "Erreur : un horaire alternatif correspond à l’horaire par \
                    défaut.")?;
                    return Ok(());
                }
                if reactions.contains(&emote) {
                    msg.reply(ctx, "Erreur : une emote horaire est dupliquée.")?;
                    return Ok(());
                }
                alt_times.push(time_to_serial(alt_time)?);
                reactions.push(emote);
            }
        } else {
            for i in 1..=3 {
                reactions.push(time_emote(time + Duration::minutes(30 * i))?);
            }
        }
        let data = ShadowrunConfirm {
            date_timestamp: date.and_hms(12, 0, 0).timestamp(),
            participants_raw_ids: participants.iter().map(|u| u.id.0).collect(),
            online,
            time: time_to_serial(time)?,
            alt_times,
        };
        let mut msg = msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| e.description("En préparation..."))
                .reactions(reactions)
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

#[throws]
fn time_to_serial(time: NaiveTime) -> HourHalf {
    let minute = time.minute();
    if minute != 0 && minute != 30 {
        bail!("unreachable: wrong minutes");
    }
    let hour_u8 = u8::try_from(time.hour())?;
    (hour_u8, minute == 30)
}

fn serial_to_time(serial: HourHalf) -> NaiveTime {
    NaiveTime::from_hms(serial.0.into(), if serial.1 { 30 } else { 0 }, 0)
}

fn refresh(ctx: &Context, msg: &mut Message, data: ShadowrunConfirm) -> AVoid {
    let runner: Role = RUNNER
        .to_role_cached(ctx)
        .ok_or_else(|| anyhow!("no role"))?;
    let ShadowrunConfirm {
        date_timestamp,
        participants_raw_ids,
        online,
        time: serial_time,
        alt_times: proposed_serial_alts,
    } = data.clone();
    let date = TZ_DEFAULT.timestamp(date_timestamp, 0).date();
    let mut participants = HashMap::new();
    let time = serial_to_time(serial_time);
    let alt_times: Vec<NaiveTime> = if proposed_serial_alts.is_empty() {
        (1..=3).map(|i| time + Duration::minutes(30 * i)).collect()
    } else {
        proposed_serial_alts
            .into_iter()
            .map(serial_to_time)
            .collect()
    };
    let alt_with_emotes = alt_times
        .iter()
        .cloned()
        .map(|t| Ok((t, time_emote(t)?)))
        .collect::<ARes<Vec<_>>>()?;
    for user_id_raw in &participants_raw_ids {
        participants.insert(
            UserId(*user_id_raw),
            ConfirmInfo {
                attendance: Pending,
                hosting: Unspecified,
                time,
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
                attendance: Confirmed,
                hosting: Unspecified,
                time,
            },
        );
    }
    for cancelling in rus("🚫")? {
        participants.modify(cancelling.id, |ConfirmInfo { hosting, time, .. }| {
            ConfirmInfo {
                attendance: Cancelled,
                hosting,
                time,
            }
        });
    }
    if !online {
        for granting in rus("🏠")? {
            participants.modify(
                granting.id,
                |ConfirmInfo {
                     attendance, time, ..
                 }| ConfirmInfo {
                    attendance,
                    hosting: Granted,
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
                    hosting: Demanded,
                    time,
                },
            );
        }
    }
    for (time, emote) in &alt_with_emotes {
        for time_changing in rus(emote)? {
            participants.modify(
                time_changing.id,
                move |ConfirmInfo {
                          attendance,
                          hosting,
                          ..
                      }| ConfirmInfo {
                    attendance,
                    hosting,
                    time: time.clone(),
                },
            );
        }
    }
    let data = encode(Embedded::EShadowrunConfirm(data))?;
    let host = host_priority(&participants);
    let host_nick = if online {
        "en ligne".to_owned()
    } else {
        format!("chez {}", host.mention())
    };
    msg.edit(ctx, |m| {
        let weekday_to_str = fr_weekday_to_str(date.weekday());
        let day_to_str = fr_day_to_str(date);
        let month_to_str = fr_month_to_str(date);
        let selected_time = hm24_format(&select_time(time, alt_times, &participants));
        m.content(format!(
            "Shadowrun : confirmation pour le {} {} {} à {} {}.",
            weekday_to_str, day_to_str, month_to_str, selected_time, host_nick
        ));
        m.embed(|e| {
            e.title("Shadowrun – Confirmation")
                .colour(runner.colour)
                .description({
                    let mut mb = MessageBuilder::new();
                    for (user_id, info) in &participants {
                        match info.attendance {
                            Confirmed => mb.push("✅"),
                            Cancelled => mb.push("🚫"),
                            Pending => mb.push("⌛"),
                        };
                        mb.mention(user_id).push(", ");
                    }
                    mb.push("\nLa prochaine séance aura lieu le ")
                        .push_bold(weekday_to_str)
                        .push(" ")
                        .push_bold(day_to_str)
                        .push(" ")
                        .push_bold(month_to_str)
                        .push(" à ")
                        .push_bold(selected_time);
                    if online {
                        mb.push(" en 💻 ").push_bold("ligne");
                    } else {
                        mb.push(" chez ").mention(host);
                    }
                    mb.push(".\nMerci de : ")
                        .push_bold("✅ confirmer 🚫 annuler");
                    if !online {
                        mb.push(".\nAccueil : ").push_bold("🏠 possible 🚩 demandé");
                    }
                    mb.push(".\nChanger l’horaire :");
                    for (time, emote) in alt_with_emotes {
                        mb.push(" ")
                            .push(emote)
                            .push(" ")
                            .push_bold(hm24_format(&time));
                    }
                    mb.push(".");
                    mb
                })
                .footer(|f| f.text(data))
        })
    })?;
    Ok(())
}

fn select_time(
    default: NaiveTime,
    alternatives: Vec<NaiveTime>,
    participants: &HashMap<UserId, ConfirmInfo>,
) -> NaiveTime {
    let asked_times: HashSet<NaiveTime> = participants
        .iter()
        .filter_map(
            |(
                _,
                ConfirmInfo {
                    attendance, time, ..
                },
            )| (attendance == &Confirmed).as_some(time),
        )
        .cloned()
        .collect();
    alternatives
        .into_iter()
        .rfind(|time| asked_times.contains(time))
        .unwrap_or(default)
}

fn host_priority(participants: &HashMap<UserId, ConfirmInfo>) -> &UserId {
    for offer in &[Demanded, Granted] {
        let hosts = participants.iter().filter_map(|(id, info)| {
            (info.attendance == Confirmed && &info.hosting == offer).as_some(id)
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
    time: NaiveTime,
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

fn last_plan(ctx: &Context, base: &Message) -> ARes<Message> {
    if let Ok((msg, _)) = find_by_state(ctx, base, |d| matches!(d, Embedded::EShadowrunPlan(_))) {
        Ok(msg)
    } else {
        base.reply(ctx, "je n’ai pas trouvé le dernier planning.")?;
        bail!("could not find plan message")
    }
}

fn read_participants_date<T: TimeZone>(
    ctx: &Context,
    plan: &Message,
    day: Weekday,
    online: bool,
    tz: T,
) -> ARes<(Vec<User>, Date<T>)> {
    let mut participants = plan.reaction_users(
        ctx,
        Unicode(fr_weekday_to_emote(day).to_owned()),
        None,
        None,
    )?;
    pop_self(ctx, &mut participants)?;
    if !online {
        let only_online: Vec<UserId> = plan
            .reaction_users(ctx, Unicode("💻".to_owned()), None, None)?
            .into_iter()
            .map(|u| u.id)
            .collect();
        participants.retain(|u| !only_online.contains(&u.id))
    }
    let mut date = plan.timestamp.with_timezone(&tz).date();
    while date.weekday() != day {
        date = date.succ();
    }
    Ok((participants, date))
}
