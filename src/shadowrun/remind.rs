use crate::{
    error::ARes,
    shadowrun::runners,
    state::{find_by_state, Embedded},
};
use anyhow::bail;
use serenity::{
    client::Context, model::channel::Message, model::channel::ReactionType::Unicode,
    model::id::UserId, utils::MessageBuilder,
};
use sparky_macros::cmd;
use std::collections::HashSet;

#[cmd]
#[description = "Analyse le précédent sondage (planning ou confirmation) et notifie les \
utilisateurs n’ayant pas voté."]
fn remind(ctx: &Context, msg: &Message) {
    let poll = last_poll(ctx, msg)?;
    let status = status(ctx, poll)?;
    let Status {
        polled: mut pending,
        answered,
    } = status;
    pending.retain(|u| !answered.contains(u));
    if pending.is_empty() {
        msg.reply(ctx, "tout le monde a voté.")?;
        return;
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
    if let Ok((msg, state)) = find_by_state(
        ctx,
        base,
        |b| matches!(b, Embedded::EShadowrunPlan(_) | Embedded::EShadowrunConfirm(_)),
    ) {
        Ok(match state {
            Embedded::EShadowrunPlan(_) => Poll {
                message: msg,
                kind: Kind::Plan,
            },
            Embedded::EShadowrunConfirm(data) => Poll {
                message: msg,
                kind: Kind::Confirm {
                    participants: data
                        .participants_raw_ids
                        .iter()
                        .map(|&id| UserId(id))
                        .collect(),
                },
            },
            _ => unreachable!("previously matched"),
        })
    } else {
        base.reply(ctx, "je n’ai pas trouvé le dernier sondage.")?;
        bail!("could not find plan message")
    }
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
