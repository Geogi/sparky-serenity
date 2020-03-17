use crate::error::ARes;
use crate::shadowrun::runners;
use crate::state::{extract, Embedded};
use anyhow::bail;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType::Unicode;
use serenity::model::id::UserId;
use serenity::utils::MessageBuilder;
use std::collections::HashSet;

#[command]
fn remind(ctx: &mut Context, msg: &Message) -> CommandResult {
    let ctx = &*ctx;
    let poll = last_poll(ctx, msg)?;
    let status = status(ctx, poll)?;
    let Status {
        polled: mut pending,
        answered,
    } = status;
    pending.retain(|u| !answered.contains(u));
    if pending.is_empty() {
        msg.reply(ctx, "tout le monde a voté.")?;
        return Ok(());
    }
    msg.channel_id.send_message(ctx, |m| {
        m.content({
            let mut mb = MessageBuilder::new();
            mb.push("Rappel : un sondage est en cours. ");
            for user in pending {
                mb.mention(&user);
                mb.push(", ");
            }
            mb.push("merci de répondre ; utiliser 🚫 si pas possible.");
            mb
        })
    })?;
    Ok(())
}

struct Status {
    polled: Vec<UserId>,
    answered: HashSet<UserId>,
}

struct Poll {
    message: Message,
    kind: Kind,
}

enum Kind {
    Plan,
    Confirm { participants: Vec<UserId> },
}

fn last_poll(ctx: &Context, base: &Message) -> ARes<Poll> {
    for msg in base.channel_id.messages(ctx, |r| r.before(base.id))? {
        if let Some(Embedded::EShadowrunPlan(_)) = extract(ctx, &msg) {
            return Ok(Poll {
                message: msg,
                kind: Kind::Plan,
            });
        }
        if let Some(Embedded::EShadowrunConfirm(data)) = extract(ctx, &msg) {
            return Ok(Poll {
                message: msg,
                kind: Kind::Confirm {
                    participants: data
                        .participants_raw_ids
                        .iter()
                        .map(|&id| UserId(id))
                        .collect(),
                },
            });
        }
    }
    base.reply(ctx, "je n’ai pas trouvé le dernier sondage.")?;
    bail!("could not find plan message")
}

fn status(ctx: &Context, poll: Poll) -> ARes<Status> {
    let Poll { message, kind } = poll;
    Ok(match kind {
        Kind::Plan => Status {
            polled: runners(ctx)?,
            answered: emote_users(ctx, &message, &["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩"])?,
        },
        Kind::Confirm { participants } => Status {
            polled: participants,
            answered: emote_users(ctx, &message, &["✅", "🚫"])?,
        },
    })
}

fn emote_users(ctx: &Context, msg: &Message, emotes: &[&str]) -> ARes<HashSet<UserId>> {
    let mut res = HashSet::new();
    for emote in emotes {
        for user in msg.reaction_users(ctx, Unicode(emote.to_owned().to_owned()), None, None)? {
            res.insert(user.id);
        }
    }
    Ok(res)
}
