use crate::date::{
    fr_day_to_str, fr_month_to_str, fr_weekday_from_str, fr_weekday_to_emote, fr_weekday_to_str,
};
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
use serenity::model::channel::{Channel, Embed, Message, Reaction};
use serenity::model::guild::Role;
use serenity::utils::MessageBuilder;
use std::collections::HashSet;
use std::ops::{BitXor, Deref};

#[derive(Serialize, Deserialize)]
pub struct ShadowrunPlan;

#[command]
pub fn plan(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ctx = ctx.deref();
    let today = Utc::today();
    let runner: Role = RUNNER.to_role_cached(ctx).ok_or(anyhow!("no role"))?;
    let last_day = today + Duration::days(6);
    let data = encode(Embedded::EShadowrunPlan(ShadowrunPlan))?;
    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Shadowrun – Prochaine séance")
                .colour(runner.colour)
                .description(
                    MessageBuilder::new()
                        .mention(&runner)
                        .push(", vos disponibilités jusqu'au ")
                        .push(fr_day_to_str(last_day))
                        .push(" ")
                        .push(fr_month_to_str(last_day))
                        .push("."),
                )
                .fields((0..=6).map(|inc| {
                    let date = today + Duration::days(inc);
                    (
                        format!(
                            "{} {}",
                            fr_weekday_to_str(date.weekday()).to_sentence_case(),
                            fr_day_to_str(date)
                        ),
                        "\u{200b}".to_owned(),
                        true,
                    )
                }))
                .footer(|f| f.text(data))
        })
        .reactions(
            (0..=6)
                .map(|inc| fr_weekday_to_emote((today + Duration::days(inc)).weekday()))
                .chain(vec!["🚫"]),
        )
    })?;
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

pub fn plan_edit(ctx: &Context, reaction: &Reaction) -> AVoid {
    if reaction.user_id != ctx.http.get_current_user()?.id {
        if let Channel::Guild(chan_cell) = reaction
            .channel_id
            .to_channel_cached(ctx)
            .ok_or(anyhow!("cannot read channel"))?
        {
            let chan = chan_cell.read();
            let mut message: Message = chan.message(ctx, reaction.message_id)?;
            if is_sr_plan(ctx, &message) {
                let orig = message
                    .embeds
                    .first()
                    .ok_or(anyhow!("cannot parse embed"))?
                    .clone();
                let guild_id = chan.guild_id;
                let mut available = vec![];
                let mut voted = HashSet::new();
                let footer = orig.clone().footer.ok_or(anyhow!("no footer"))?.text;
                let wd_str = &orig
                    .fields
                    .first()
                    .ok_or(anyhow!("no 1st field"))?
                    .name
                    .split(' ')
                    .next()
                    .ok_or(anyhow!("no 1st part of 1st field"))?;
                let mut day = fr_weekday_from_str(wd_str).ok_or(anyhow!("cannot parse weekday"))?;
                for _ in 0..=6 {
                    let mut users = message.reaction_users(
                        ctx,
                        Unicode(fr_weekday_to_emote(day).to_owned()),
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
                for user in message.reaction_users(ctx, Unicode("🚫".to_owned()), None, None)? {
                    voted.insert(user.id);
                }
                let guild_cell = guild_id
                    .to_guild_cached(ctx)
                    .ok_or(anyhow!("cannot read guild"))?;
                let guild = guild_cell.read();
                let mut runners = guild.members.iter().filter_map(|(id, member)| {
                    if member.roles.contains(&RUNNER) {
                        Some(id)
                    } else {
                        None
                    }
                });
                let exhaustive = runners.all(|id| voted.contains(id));
                let Embed {
                    title: m_title,
                    description: m_description,
                    colour,
                    fields,
                    ..
                } = orig;
                let orig_title = m_title.ok_or(anyhow!("cannot read embed title"))?;
                let orig_description =
                    m_description.ok_or(anyhow!("cannot read embed description"))?;
                message.edit(ctx, |m| {
                    m.embed(|e| {
                        e.title(orig_title)
                            .description(maybe_check(orig_description, exhaustive))
                            .colour(colour)
                            .footer(|f| f.text(footer))
                            .fields(fields.iter().enumerate().map(|(inc, field)| {
                                (
                                    field.name.clone(),
                                    {
                                        let list = &available[inc];
                                        if list.is_empty() {
                                            "\u{200b}".to_owned()
                                        } else {
                                            list.iter()
                                                .map(|user| {
                                                    user.nick_in(ctx, guild_id)
                                                        .unwrap_or(user.name.clone())
                                                })
                                                .collect::<Vec<String>>()
                                                .join("\n")
                                        }
                                    },
                                    true,
                                )
                            }))
                    })
                })?;
            }
        }
    }
    Ok(())
}

fn maybe_check(mut description: String, exhaustive: bool) -> String {
    if !description.starts_with("✅").bitxor(exhaustive) {
        description
    } else if exhaustive {
        let mut out = String::from("✅ ");
        out.push_str(&description);
        out
    } else {
        description.drain(..4);
        description
    }
}
