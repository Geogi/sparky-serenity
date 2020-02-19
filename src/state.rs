use crate::shadowrun::ShadowrunPlan;
use base64::write::EncoderWriter;
use base64::STANDARD;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use snap::read::FrameDecoder;
use snap::write::FrameEncoder;
use std::io::{Read, Write};
use std::fs::read;

const CHUNK_LEN: usize = 60;

#[non_exhaustive]
#[derive(Serialize, Deserialize)]
pub enum Embedded {
    EShadowrunPlan(ShadowrunPlan),
}

pub fn encode(input: Embedded) -> String {
    let bin = serialize(&input).unwrap();
    let mut buf = vec![];
    {
        let mut base = EncoderWriter::new(&mut buf, STANDARD);
        let mut snap = FrameEncoder::new(&mut base);
        snap.write_all(&bin).unwrap();
    }
    let mut based = String::from_utf8(buf).unwrap();
    let mut split = String::new();
    while based.len() > CHUNK_LEN {
        let drain: String = based.drain(..CHUNK_LEN).collect();
        split.push_str(&drain);
        split.push('\n');
    }
    split.push_str(&based);
    split
}

pub fn decode(input: &str) -> Option<Embedded> {
    let no_split = input.replace("\n", "");
    let un_base = base64::decode(&no_split).unwrap();
    let mut un_snap = FrameDecoder::new(un_base.as_slice());
    let mut buf = vec![];
    un_snap.read_to_end(&mut buf).unwrap();
    deserialize::<Embedded>(&buf).ok()
}
