use crate::error::wrap_cmd_err;
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

#[derive(Deserialize)]
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
        let api_url = Url::parse(FFLOGS_API_V1)?;
        let mut url = api_url;
        url.path_segments_mut()
            .map_err(|_| anyhow!("cannot-be-a-base"))?
            .push("parses")
            .push("character")
            .push(character)
            .push(server.unwrap_or("Omega"))
            .push("EU");
        url.query_pairs_mut()
            .append_pair("api_key", &api_key)
            .append_pair("metric", "rdps")
            .append_pair("timeframe", "historical");
        let result = crate::http::get(ctx, url)?;
        if result.status() == StatusCode::BAD_REQUEST {
            msg.reply(ctx, "FFLogs ne trouve pas ce personnage.")?;
            return Ok(());
        }
        let body = result.text()?;
        let all_parses: Vec<Parse> = serde_json::from_str(&body)?;
        let mut best_by_encounter = HashMap::new();
        for parse in &all_parses {
            if best_by_encounter
                .get(&(parse.encounterID, parse.difficulty))
                .map(|(_, p)| p < &parse.percentile)
                .unwrap_or(true)
            {
                best_by_encounter.insert(
                    (parse.encounterID, parse.difficulty),
                    (parse.reportID.clone(), parse.percentile),
                );
            }
        }
        let best_logs = all_parses.into_iter().filter(|p| {
            best_by_encounter
                .get(&(p.encounterID, p.difficulty))
                .unwrap()
                .0
                == p.reportID
        });
        let mut mb = MessageBuilder::new();
        for log in best_logs {
            mb.push(&log.encounterName);
            if log.difficulty == 101 {
                mb.push(" (Sadique)");
            }
            mb.push(" : ");
            mb.push_italic(&log.spec);
            mb.push(" ");
            mb.push_bold_line(format!("{:.0}%", log.percentile.floor()));
        }
        msg.channel_id
            .send_message(ctx, |m| m.embed(|e| e.title(character).description(mb)))?;
        Ok(())
    })
}

fn char_name(input: &str) -> IResult<&str, &str> {
    let end = input.find(|c: char| !c.is_alpha() && ![' ', '\'', '-'].contains(&c));
    let (matched, rest) = if let Some(end) = end {
        input.split_at(end)
    } else {
        (input, "")
    };
    Ok((rest, matched))
}
