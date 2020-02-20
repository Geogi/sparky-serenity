use crate::date::{fr_day, fr_month, fr_weekday, weekday_emote};
use crate::state::{decode, encode, Embedded};
use chrono::{Datelike, Duration, Utc, Weekday};
use inflector::Inflector;
use serde::Deserialize;
use serde::Serialize;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::channel::{Channel, Message, Reaction};
use serenity::model::guild::Role;
use serenity::model::id::RoleId;
use serenity::utils::MessageBuilder;

match_id! {
const RUNNER: RoleId = match {
    exylobby => 293393770941251584,
    ytp => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[commands(plan)]
pub struct Shadowrun;

pub fn shadowrun_reaction_add(ctx: Context, add_reaction: Reaction) {
    plan_edit(ctx, add_reaction);
}

pub fn shadowrun_reaction_remove(ctx: Context, removed_reaction: Reaction) {
    plan_edit(ctx, removed_reaction);
}

#[derive(Serialize, Deserialize)]
pub struct ShadowrunPlan {
    first_day: Weekday,
}

#[command]
fn plan(ctx: &mut Context, msg: &Message) -> CommandResult {
    let today = Utc::today();
    let runner: Role = RUNNER.to_role_cached(&ctx).unwrap();
    let last_day = today + Duration::days(6);
    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Shadowrun – Prochaine séance")
                    .colour(runner.colour)
                    .description(
                        MessageBuilder::new()
                            .mention(&runner)
                            .push(", vos disponibilités jusqu'au ")
                            .push(last_day.day())
                            .push(" ")
                            .push(fr_month(last_day))
                            .push("."),
                    )
                    .fields((0..=6).map(|inc| {
                        let date = today + Duration::days(inc);
                        (
                            format!("{} {}", fr_weekday(date).to_sentence_case(), fr_day(date)),
                            "\u{200b}".to_owned(),
                            true,
                        )
                    }))
                    .footer(|f| {
                        f.text(encode(Embedded::EShadowrunPlan(ShadowrunPlan {
                            first_day: today.weekday(),
                        })))
                    })
            })
            .reactions(
                (0..=6)
                    .map(|inc| weekday_emote((today + Duration::days(inc)).weekday()))
                    .chain(vec!["🚫"]),
            )
        })
        .unwrap();
    Ok(())
}

fn plan_edit(ctx: Context, reaction: Reaction) {
    if reaction.user_id != ctx.http.get_current_user().unwrap().id {
        if let Channel::Guild(chan_cell) = reaction.channel_id.to_channel_cached(&ctx).unwrap() {
            let chan = chan_cell.read();
            let mut message: Message = chan.message(&ctx, reaction.message_id).unwrap();
            if message.is_own(&ctx) {
                if let Some(embed) = message.embeds.first() {
                    if let Some(footer) = &embed.footer {
                        if let Some(Embedded::EShadowrunPlan(ShadowrunPlan { first_day, .. })) =
                            decode(&footer.text)
                        {
                            let orig = message.embeds.first().unwrap().clone();
                            let guild = chan.guild_id;
                            let mut available = vec![];
                            let mut day = first_day;
                            let footer = orig.clone().footer.unwrap().text;
                            for _ in 0..=6 {
                                let mut users = message
                                    .reaction_users(
                                        &ctx,
                                        Unicode(weekday_emote(day).to_owned()),
                                        None,
                                        None,
                                    )
                                    .unwrap();
                                users.retain(|u| u.id != ctx.http.get_current_user().unwrap().id);
                                available.push(users);
                                day = day.succ();
                            }
                            message
                                .edit(&ctx, |m| {
                                    m.embed(|e| {
                                        e.title(orig.title.unwrap())
                                            .description(orig.description.unwrap())
                                            .colour(orig.colour)
                                            .footer(|f| f.text(footer))
                                            .fields(orig.fields.iter().enumerate().map(
                                                |(inc, field)| {
                                                    (
                                                        field.name.clone(),
                                                        {
                                                            let list = available
                                                                .get(inc as usize)
                                                                .unwrap();
                                                            if list.is_empty() {
                                                                "\u{200b}".to_owned()
                                                            } else {
                                                                list.iter()
                                                                    .map(|user| {
                                                                        user.nick_in(&ctx, guild)
                                                                            .unwrap_or(
                                                                                user.name.clone(),
                                                                            )
                                                                    })
                                                                    .collect::<Vec<String>>()
                                                                    .join("\n")
                                                            }
                                                        },
                                                        true,
                                                    )
                                                },
                                            ))
                                    })
                                })
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
}
