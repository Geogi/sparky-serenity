mod songs;

use crate::error::wrap_cmd_err;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use typemap::Key;
use chrono::{Utc, DateTime, Duration};
use crate::edf::songs::SONGS;
use anyhow::Context as _;

#[group]
#[prefix = "edf"]
#[commands(sing)]
pub struct Edf;

struct SongFollowUp {
    last_time: DateTime<Utc>,
    song_id: usize,
}

const FOLLOWUP_INTERVAL: i64 = 30;

struct SongFollowUpKey;
impl Key for SongFollowUpKey {
    type Value = SongFollowUp;
}

#[command]
fn sing(ctx: &mut Context, msg: &Message) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let followup = {
            let data = ctx.data.read();
            data.get::<SongFollowUpKey>()
                .filter(|sf| Utc::now() - sf.last_time < Duration::seconds(FOLLOWUP_INTERVAL))
                .map(|sf| sf.song_id)
        };
        let next = followup.map(|id| (id + 1) % SONGS.len()).unwrap_or(0);
        {
            let mut data = ctx.data.write();
            data.insert::<SongFollowUpKey>(SongFollowUp {
                last_time: Utc::now(),
                song_id: next,
            });
        }
            let song = SONGS.get(next).context("wrong song id")?;
            msg.channel_id.send_message(ctx, |m|
                m.content(song))?;
        Ok(())
    })
}
