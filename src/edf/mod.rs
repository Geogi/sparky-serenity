mod songs;

use crate::error::wrap_cmd_err;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;
use typemap::Key;

#[group]
#[prefix = "edf"]
#[commands(sing)]
pub struct Edf;

struct SongFollowUp {
    last_timestamp: u64,
    song_id: u8,
}

struct SongFollowUpKey;
impl Key for SongFollowUpKey {
    type Value = SongFollowUp;
}

#[command]
fn sing(ctx: &mut Context, msg: &Message) -> CommandResult {
    wrap_cmd_err(|| {
        Ok(())
    })
}
