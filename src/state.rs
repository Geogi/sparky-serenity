use crate::edf::EdfSing;
use crate::error::ARes;
use crate::shadowrun::confirm::ShadowrunConfirm;
use crate::shadowrun::plan::ShadowrunPlan;
use crate::utils::{find_message_with, find_message_with_limit};
use base64::write::EncoderWriter;
use base64::STANDARD;
use bincode::{deserialize, serialize};
use boolinator::Boolinator;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serenity::client::Context;
use serenity::model::channel::Message;
use std::io::{Read, Write};

const CHUNK_LEN: usize = 60;

#[non_exhaustive]
#[derive(Serialize, Deserialize)]
pub enum Embedded {
    EShadowrunPlan(ShadowrunPlan),
    EShadowrunConfirm(ShadowrunConfirm),
    EEdfSing(EdfSing),
}

pub fn encode(input: Embedded) -> ARes<String> {
    let bin = serialize(&input)?;
    let mut buf = vec![];
    {
        let mut base = EncoderWriter::new(&mut buf, STANDARD);
        let mut snap = GzEncoder::new(&mut base, Compression::best());
        snap.write_all(&bin)?;
    }
    let mut based = String::from_utf8(buf)?;
    let mut split = String::new();
    while based.len() > CHUNK_LEN {
        let drain: String = based.drain(..CHUNK_LEN).collect();
        split.push_str(&drain);
        split.push('\n');
    }
    split.push_str(&based);
    Ok(split)
}

pub fn decode(input: &str) -> Option<Embedded> {
    let no_split = input.replace("\n", "");
    if let Ok(un_base) = base64::decode(&no_split) {
        let mut un_snap = GzDecoder::new(un_base.as_slice());
        let mut buf = vec![];
        if un_snap.read_to_end(&mut buf).is_ok() {
            let a = deserialize::<Embedded>(&buf);
            return a.ok();
        }
    }
    None
}

pub fn extract(ctx: &Context, message: &Message) -> Option<Embedded> {
    if message.is_own(ctx) {
        if let Some(embed) = message.embeds.first() {
            if let Some(footer) = &embed.footer {
                return decode(&footer.text);
            }
        }
    }
    None
}

pub fn find_by_state(
    ctx: &Context,
    base: &Message,
    mut pred: impl FnMut(&Embedded) -> bool,
) -> ARes<(Message, Embedded)> {
    find_message_with(ctx, base, |msg| {
        extract(ctx, msg).and_then(|state| pred(&state).as_some(state))
    })
}

pub fn find_by_state_limit(
    ctx: &Context,
    base: &Message,
    mut pred: impl FnMut(&Embedded) -> bool,
    limit: usize,
) -> ARes<(Message, Embedded)> {
    find_message_with_limit(
        ctx,
        base,
        |msg| extract(ctx, msg).and_then(|state| pred(&state).as_some(state)),
        limit,
    )
}
