use crate::error::wrap_cmd_err;
use crate::string::StrExt;
use anyhow::{anyhow, Context as _};
use nom::character::complete::{alpha1, char};
use nom::{
    combinator::opt,
    sequence::{preceded, tuple},
    AsChar, IResult,
};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    utils::MessageBuilder,
};
use std::collections::HashMap;

const FFLOGS_API_V1: &str = "https://www.fflogs.com/v1";

const ZONES: &[(&[u64], bool, &str)] = &[
    (&[28], false, "Défis (Extrême)"),
    (&[29], true, "Eden’s Gate (Sadique)"),
    (&[33], true, "Eden’s Verse (Sadique)"),
    (&[19, 23, 30, 32], false, "Fatals"),
];

#[derive(Deserialize)]
#[allow(dead_code, non_snake_case)]
struct Ranking {
    encounterID: u64,
    encounterName: String,
    class: String,
    spec: String,
    rank: u64,
    outOf: u64,
    duration: u64,
    startTime: u64,
    reportID: String,
    fightID: u64,
    difficulty: u64,
    characterID: u64,
    characterName: String,
    server: String,
    percentile: f64,
    ilvlKeyOrPatch: f64,
    total: f64,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code, non_snake_case)]
struct Parse {
    encounterID: u64,
    encounterName: String,
    class: String,
    spec: String,
    rank: u64,
    outOf: u64,
    duration: u64,
    startTime: u64,
    reportID: String,
    fightID: u64,
    difficulty: u64,
    characterID: u64,
    characterName: String,
    server: String,
    percentile: f64,
    ilvlKeyOrPatch: f64,
    total: f64,
    estimated: bool,
}

#[command]
#[description = "Récupère les meilleurs rangs historiques depuis FFLogs"]
pub fn bestlogs(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx;
        let (character, server) = if let Ok(("", res)) =
            { tuple((char_name, opt(preceded(char('@'), alpha1))))(_args.message()) }
        {
            res
        } else {
            msg.reply(ctx, "Commande invalide.")?;
            return Ok(());
        };
        let api_key = std::env::var("FFLOG_V1_KEY").context("no fflogs api key in env")?;
        let mut api_base = Url::parse(FFLOGS_API_V1)?;
        api_base
            .path_segments_mut()
            .map_err(|_| anyhow!("cannot-be-a-base"))?
            .push("parses")
            .push("character")
            .push(&character)
            .push(server.unwrap_or("Omega"))
            .push("EU");
        let mut mbs = vec![];
        for kind in ZONES {
            let mut kind_parses = vec![];
            let mut mb = MessageBuilder::new();
            for zone in kind.0 {
                let mut api_zone = api_base.clone();
                api_zone
                    .query_pairs_mut()
                    .append_pair("api_key", &api_key)
                    .append_pair("timeframe", "historical")
                    .append_pair("zone", &zone.to_string());
                if zone >= &28 {
                    api_zone.query_pairs_mut().append_pair("metric", "rdps");
                }
                let result = crate::http::get(ctx, api_zone)?;
                if result.status() == StatusCode::BAD_REQUEST {
                    msg.reply(ctx, "FFLogs ne trouve pas ce personnage.")?;
                    return Ok(());
                }
                let body = result.text()?;
                let all_parses: Vec<Parse> = serde_json::from_str(&body)?;
                let mut encounter_perc = HashMap::new();
                let mut encounter_report = HashMap::new();
                for parse in all_parses
                    .iter()
                    .filter(|p| p.difficulty == if kind.1 { 101 } else { 100 })
                {
                    if encounter_perc
                        .get(&parse.encounterID)
                        .map(|p| p < &parse.percentile)
                        .unwrap_or(true)
                    {
                        encounter_perc.insert(parse.encounterID, parse.percentile);
                        encounter_report.insert(parse.encounterID, parse);
                    }
                }
                let mut sorted: Vec<Parse> = encounter_report.values().cloned().cloned().collect();
                sorted.sort_by_key(|e| e.encounterID);
                kind_parses.append(&mut sorted);
            }
            for log in &kind_parses {
                mb.push(&log.encounterName)
                    .push(" : ")
                    .push_italic(&log.spec)
                    .push(" ")
                    .push_bold_line(format!("{:.0}%", log.percentile.floor()));
            }
            if !kind_parses.is_empty() {
                mbs.push((kind.2, mb, false));
            }
        }
        msg.channel_id.send_message(ctx, |m| {
            m.embed(|e| e.title(character.title_case()).fields(mbs))
        })?;
        Ok(())
    })
}

fn char_name(input: &str) -> IResult<&str, String> {
    let end = input.find(|c: char| !c.is_alpha() && ![' ', '\'', '’', '-'].contains(&c));
    let (matched, rest) = if let Some(end) = end {
        input.split_at(end)
    } else {
        (input, "")
    };
    let correct_apostrophes = matched.replace('’', "'");
    Ok((rest, correct_apostrophes))
}
