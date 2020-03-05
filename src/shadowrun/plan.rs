use crate::date::{fr_day_to_str, fr_month_to_str, fr_weekday_to_emote, fr_weekday_to_str};
use crate::error::AVoid;
use crate::shadowrun::RUNNER;
use crate::state::{decode, encode, Embedded};
use crate::utils::pop_self;
use anyhow::anyhow;
use chrono::{Datelike, Duration, Utc};
use inflector::Inflector;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::channel::{Message, Reaction};
use serenity::model::guild::Role;
use serenity::utils::MessageBuilder;
use std::collections::HashSet;
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
pub struct ShadowrunPlan;

#[command]
pub fn plan(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ctx = ctx.deref();
    let first_day = msg.timestamp.with_timezone(&Utc).date();
    let mut base = msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| e.description("En préparation...")).reactions(
            (0..=6)
                .map(|inc| fr_weekday_to_emote((first_day + Duration::days(inc)).weekday()))
                .chain(vec!["🚫"]),
        )
    })?;
    refresh(ctx, &mut base)?;
    Ok(())
}

pub fn plan_react(ctx: &Context, reaction: &Reaction) -> AVoid {
    let mut msg = reaction.message(ctx)?;
    if is_sr_plan(ctx, &msg) {
        refresh(ctx, &mut msg)?;
    }
    Ok(())
}

pub fn is_sr_plan(ctx: &Context, message: &Message) -> bool {
    if message.is_own(ctx) {
        if let Some(embed) = message.embeds.first() {
            if let Some(footer) = &embed.footer {
                if let Some(Embedded::EShadowrunPlan(ShadowrunPlan)) = decode(&footer.text) {
                    return true;
                }
            }
        }
    }
    return false;
}

fn refresh(ctx: &Context, msg: &mut Message) -> AVoid {
    let runner: Role = RUNNER.to_role_cached(ctx).ok_or(anyhow!("no role"))?;
    let first_day = msg.timestamp.with_timezone(&Utc).date();
    let last_day = first_day + Duration::days(6);
    let guild = msg.guild(ctx).ok_or(anyhow!("cannot get guild"))?;
    let guild_id = guild.read().id;
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
    let guild = guild.read();
    let mut runners = guild.members.iter().filter_map(|(id, member)| {
        if member.roles.contains(&RUNNER) {
            Some(id)
        } else {
            None
        }
    });
    let exhaustive = runners.all(|id| voted.contains(id));
    let data = encode(Embedded::EShadowrunPlan(ShadowrunPlan))?;
    msg.edit(ctx, |m| {
        m.embed(|e| {
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
                        .push(fr_day_to_str(last_day))
                        .push(" ")
                        .push(fr_month_to_str(last_day))
                        .push(".");
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
                                        user.nick_in(ctx, guild_id).unwrap_or(user.name.clone())
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
