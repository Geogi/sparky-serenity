use crate::date::{fr_day, fr_month, fr_weekday, weekday_emote};
use crate::handler::Handler;
use crate::state::{decode, encode, Embedded};
use chrono::{Date, DateTime, Datelike, Duration, NaiveDateTime, Utc};
use inflector::Inflector;
use serde::Deserialize;
use serde::Serialize;
use serenity::builder::CreateEmbed;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::channel::{Channel, Message, Reaction};
use serenity::model::guild::Role;
use serenity::model::id::{ChannelId, GuildId, RoleId, UserId};
use serenity::utils::MessageBuilder;
use std::collections::HashSet;

match_id! {
const RUNNER: RoleId = match {
    exylobby => 293393770941251584,
    ytp => 679702431222726715,
}}

#[group]
#[prefix = "sr"]
#[commands(plan)]
pub struct Shadowrun;

pub fn shadowrun_reaction_add(handler: &Handler, ctx: Context, add_reaction: Reaction) {
    plan_reaction_add(handler, ctx, add_reaction);
}

pub fn shadowrun_reaction_remove(handler: &Handler, ctx: Context, removed_reaction: Reaction) {
    plan_reaction_remove(handler, ctx, removed_reaction);
}

#[derive(Serialize, Deserialize)]
pub struct ShadowrunPlan {
    first_day_timestamp: i64,
    available: Vec<HashSet<UserId>>,
    unavailable: HashSet<UserId>,
}

#[command]
fn plan(ctx: &mut Context, msg: &Message) -> CommandResult {
    let today = Utc::today();
    let plan = ShadowrunPlan {
        first_day_timestamp: today.and_hms(0, 0, 0).timestamp(),
        available: (0..7).map(|_| HashSet::new()).collect(),
        unavailable: HashSet::new(),
    };
    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| plan_embed(&ctx, e, plan, msg.guild_id.unwrap()))
                .reactions(
                    (0..=6)
                        .map(|inc| weekday_emote(today + Duration::days(inc)))
                        .chain(vec!["🚫"]),
                )
        })
        .unwrap();
    Ok(())
}

fn plan_embed<'a>(
    ctx: &Context,
    e: &'a mut CreateEmbed,
    plan: ShadowrunPlan,
    guild: GuildId,
) -> &'a mut CreateEmbed {
    let runner: Role = RUNNER.to_role_cached(&ctx).unwrap();
    let first_day = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(plan.first_day_timestamp, 0),
        Utc,
    )
    .date();
    let last_day = first_day + Duration::days(6);
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
            let date = first_day + Duration::days(inc);
            (
                format!("{} {}", fr_weekday(date).to_sentence_case(), fr_day(date)),
                {
                    let list = plan.available.get(inc as usize).unwrap();
                    if list.is_empty() {
                        "\u{200b}".to_owned()
                    } else {
                        list.iter()
                            .map(|id| {
                                let user_cell = id.to_user_cached(&ctx).unwrap();
                                let user = user_cell.read();
                                user.nick_in(ctx, guild).unwrap_or(user.name.clone())
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    }
                },
                true,
            )
        }))
        .footer(|f| f.text(encode(Embedded::EShadowrunPlan(plan))))
}

fn plan_reaction_add(_handler: &Handler, ctx: Context, add_reaction: Reaction) {
    if add_reaction.user_id != ctx.http.get_current_user().unwrap().id {
        if let Channel::Guild(chan_cell) = add_reaction.channel_id.to_channel_cached(&ctx).unwrap()
        {
            let chan = chan_cell.read();
            let mut message: Message = chan.message(&ctx, add_reaction.message_id).unwrap();
            if message.is_own(&ctx) {
                if let Some(embed) = message.embeds.first() {
                    if let Some(footer) = &embed.footer {
                        if let Some(Embedded::EShadowrunPlan(mut plan)) = decode(&footer.text) {
                            if let Unicode(em) = add_reaction.emoji {
                                if em == "🚫" {
                                    plan.unavailable.insert(add_reaction.user_id);
                                }
                            }
                            message
                                .edit(&ctx, |m| {
                                    m.embed(|e| plan_embed(&ctx, e, plan, chan.guild_id))
                                })
                                .unwrap();
                        }
                    }
                }
            }
        }
    }
}

fn plan_reaction_remove(_handler: &Handler, _ctx: Context, _removed_reaction: Reaction) {}
