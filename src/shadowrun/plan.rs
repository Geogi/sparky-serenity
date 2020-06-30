use crate::{
    date::{fr_day_to_str, fr_month_to_str, fr_weekday_to_emote, fr_weekday_to_str, TZ_DEFAULT},
    discord::{pop_self, reaction_is_own},
    error::{wrap_cmd_err, AVoid},
    shadowrun::{runners, RUNNER},
    state::{encode, extract, Embedded},
};
use anyhow::anyhow;
use chrono::{Datelike, Duration};
use inflector::Inflector;
use serde::{Deserialize, Serialize};
use serenity::{
    client::Context,
    framework::standard::macros::command,
    framework::standard::{Args, CommandResult},
    model::channel::ReactionType::Unicode,
    model::channel::{Message, Reaction},
    model::guild::Role,
    utils::MessageBuilder,
};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct ShadowrunPlan;

#[command]
#[description = "Crée un planning jusqu’à la semaine suivante."]
pub fn plan(ctx: &mut Context, msg: &Message, mut _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let first_day = msg.timestamp.with_timezone(&TZ_DEFAULT).date();
        let runners = runners(ctx)?;
        let mut base = msg.channel_id.send_message(ctx, |m| {
            m.content({
                let mut mb = MessageBuilder::new();
                for runner in &runners {
                    mb.mention(runner);
                    mb.push(" ");
                }
                mb
            })
            .embed(|e| e.description("En préparation..."))
            .reactions(
                (0..=6)
                    .map(|inc| fr_weekday_to_emote((first_day + Duration::days(inc)).weekday()))
                    .chain(vec!["🚫", "💻"]),
            )
        })?;
        refresh(ctx, &mut base)?;
        Ok(())
    })
}

pub fn react(ctx: &Context, reaction: &Reaction) -> AVoid {
    if reaction_is_own(ctx, reaction)? {
        return Ok(());
    }
    let mut msg = reaction.message(ctx)?;
    if let Some(Embedded::EShadowrunPlan(_)) = extract(ctx, &msg) {
        refresh(ctx, &mut msg)?;
    }
    Ok(())
}

fn refresh(ctx: &Context, msg: &mut Message) -> AVoid {
    let runner: Role = RUNNER
        .to_role_cached(ctx)
        .ok_or_else(|| anyhow!("no role"))?;
    let first_day = msg.timestamp.with_timezone(&TZ_DEFAULT).date();
    let last_day = first_day + Duration::days(6);
    let chan = msg
        .channel_id
        .to_channel(ctx)?
        .guild()
        .ok_or_else(|| anyhow!("cannot get chan"))?;
    let guild = chan
        .read()
        .guild(ctx)
        .ok_or_else(|| anyhow!("cannot get guild"))?;
    let guild_id = guild.read().id;
    let online = msg.reaction_users(ctx, Unicode("💻".to_owned()), None, None)?;
    let mut available = vec![];
    let mut voted = HashSet::new();
    let mut day = first_day;
    for _ in 0..=6 {
        let mut users = msg.reaction_users(
            ctx,
            Unicode(fr_weekday_to_emote(day.weekday()).to_owned()),
            None,
            None,
        )?;
        pop_self(ctx, &mut users)?;
        for user in &users {
            voted.insert(user.id);
        }
        available.push(users);
        day = day.succ();
    }
    for user in msg.reaction_users(ctx, Unicode("🚫".to_owned()), None, None)? {
        voted.insert(user.id);
    }
    let runners = runners(ctx)?;
    let exhaustive = runners.iter().all(|id| voted.contains(id));
    let data = encode(Embedded::EShadowrunPlan(ShadowrunPlan))?;
    msg.edit(ctx, |m| {
        m.content({
            let mut mb = MessageBuilder::new();
            for runner in &runners {
                mb.mention(runner);
                mb.push(" ");
            }
            mb
        })
        .embed(|e| {
            e.title("Shadowrun – Prochaine séance")
                .colour(runner.colour)
                .description({
                    let mut mb = MessageBuilder::new();
                    if exhaustive {
                        mb.push("✅ ");
                    } else {
                        mb.push("⌛ ");
                    }
                    mb.mention(&runner)
                        .push(", vos disponibilités jusqu'au ")
                        .push_bold(fr_day_to_str(last_day))
                        .push(" ")
                        .push_bold(fr_month_to_str(last_day))
                        .push(".\nMettre 💻 si disponible uniquement en ligne.\n")
                        .push("Pensez à 🚫 si pas de disponibilité de la semaine.");
                    mb
                })
                .fields((0..=6).map(|inc| {
                    let date = first_day + Duration::days(inc);
                    (
                        format!(
                            "{} {}",
                            fr_weekday_to_str(date.weekday()).to_sentence_case(),
                            fr_day_to_str(date)
                        ),
                        {
                            let list = &available[inc as usize];
                            if list.is_empty() {
                                "\u{200b}".to_owned()
                            } else {
                                list.iter()
                                    .map(|user| {
                                        format!(
                                            "{}{}",
                                            user.nick_in(ctx, guild_id)
                                                .unwrap_or_else(|| user.name.clone()),
                                            if online.contains(user) { "·💻" } else { "" }
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("\n")
                            }
                        },
                        true,
                    )
                }))
                .footer(|f| f.text(data))
        })
    })?;
    Ok(())
}
