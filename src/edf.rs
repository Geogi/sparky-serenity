use crate::date::TZ_DEFAULT;
use crate::discord::delete_command_ifp;
use crate::error::wrap_cmd_err;
use crate::state::{encode, find_by_state_limit, Embedded};
use anyhow::{bail, Context as _};
use chrono::{Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::utils::MessageBuilder;

#[group]
#[prefix = "edf"]
#[commands(sing)]
pub struct Edf;

#[derive(Serialize, Deserialize)]
pub struct EdfSing {
    // Vec<UserId>
    verses: Vec<u64>,
    until_timestamp: i64,
}

const CHORUS_INTERVAL: i64 = 5;

#[command]
#[description = "Chante l'hymne des Forces de Défense Terrestre."]
fn sing(ctx: &mut Context, msg: &Message) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let now_ts = Utc::now().timestamp();
        let (mut target, mut state) = if let Ok((target, embed)) = find_by_state_limit(
            ctx,
            msg,
            |d| {
                matches!(d, Embedded::EEdfSing(EdfSing {until_timestamp, verses})
            if &now_ts <= until_timestamp && verses.len() < 8)
            },
            200,
        ) {
            match embed {
                Embedded::EEdfSing(state) => {
                    if state.verses.last().context("should not be empty")? == &msg.author.id.0 {
                        msg.reply(ctx, "Il est défendu de chanter seul, soldat !")?;
                        return Ok(());
                    }
                    (target, state)
                }
                _ => bail!("unreachable"),
            }
        } else {
            let target = msg
                .channel_id
                .send_message(ctx, |m| m.embed(|e| e.description("En préparation...")))?;
            let state = EdfSing {
                verses: vec![],
                until_timestamp: 0,
            };
            (target, state)
        };
        let until = (Utc::now() + Duration::minutes(CHORUS_INTERVAL + 1))
            .with_second(0)
            .context("unreachable")?;
        state.until_timestamp = until.timestamp();
        state.verses.push(msg.author.id.0);
        let guild_id = msg.guild(ctx).context("no guild")?.read().id;
        let mut description = MessageBuilder::new();
        for (index, user_id) in state.verses.iter().enumerate() {
            let user = UserId(*user_id).to_user(ctx)?;
            description
                .push_bold(user.nick_in(ctx, guild_id).unwrap_or(user.name))
                .push(" ")
                .push(SONGS[index])
                .push("\n");
        }
        if state.verses.len() < 8 {
            description
                .push_italic("\nLe chœur se termine à ")
                .push_italic(until.with_timezone(&TZ_DEFAULT).format("%H:%M"))
                .push_italic(".");
        } else {
            description.push_italic("🎌 EDF ! EDF ! 🎌");
        }
        let footer = encode(Embedded::EEdfSing(state))?;
        target.edit(ctx, |m| {
            m.embed(|e| {
                e.title("Hymne des Forces de Défense Terrestre")
                    .description(description)
                    .footer(|f| f.text(footer))
            })
        })?;
        delete_command_ifp(ctx, msg)?;
        Ok(())
    })
}

static SONGS: [&str; 8] = [
    "Oh~ We are the valiant infantry. We are the alpha team with passion and camaraderie.",
    "Hear us as we shout at the top of our lungs. Be calm and raise your guns.",
    "It's only with our sacrifice, that mankind can still exist down here in paradise.",
    "To defend our dearest Mother Earth, we're ready to give up our lives!",
    "High up in the air our comrades fight, dashing through the sky now like a million bolt of light.",
    "We shall spread our wings wide and fly high. Soaring, gliding through the endless sky!",
    "Now pick up our weapons, off we go. Firing at the flying saucers, shooting down our foe.",
    "We shall not allow these aliens, to rule homo sapiens.",
];
