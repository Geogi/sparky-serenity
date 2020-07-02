use serenity::{client::Context, framework::standard::{Args, macros::command, CommandResult}, model::channel::Message, utils::MessageBuilder};
use crate::error::wrap_cmd_err;
use serde::Deserialize;
use anyhow::{anyhow, Context as _};
use reqwest::{StatusCode, Url};
use nom::{sequence::{preceded, tuple}, IResult, AsChar, combinator::opt};
use nom::character::complete::{alpha1, char};

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

#[command]
#[description = "Récupère les rangs depuis FFLogs"]
pub fn parse(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let ctx = &*ctx; 
        let (character, server) = if let Ok(("", res)) = {
            tuple((
                char_name,
                opt(preceded(char('@'),alpha1))
            ))(_args.message())
        } {
            res
        } else {
            msg.reply(ctx, "Commande invalide.")?;
            return Ok(());
        };
        let api_key = std::env::var("FFLOG_V1_KEY").context("no fflogs api key in env")?;
        let api_url = Url::parse(FFLOGS_API_V1)?;
        let mut url = api_url.clone();
        url
        .path_segments_mut().map_err(|_| anyhow!("cannot-be-a-base"))?
        .push("rankings")
        .push("character")
        .push(character)
        .push(server.unwrap_or("Omega"))
        .push("EU");
        url
        .query_pairs_mut().append_pair("api_key", &api_key);
        let result = crate::http::get(ctx, url)?;
        if result.status() == StatusCode::BAD_REQUEST {
            msg.reply(ctx, "FFLogs ne trouve pas ce personnage.")?;
            return Ok(());
        }
        let body = result.text()?;
        let rankings: Vec<Ranking> = serde_json::from_str(&body)?;
        let mut mb = MessageBuilder::new();
        for r in rankings {
            mb.push(r.encounterName);
            if r.difficulty == 101 {
                mb.push(" (Extrême)");
            }
            mb.push(" ");
            mb.push(r.spec);
            mb.push(" ");
            mb.push_bold_line(format!("{:.1}%", r.percentile));
        }
        msg.channel_id.send_message(ctx, |m| m.embed(|e| e.title(character).description(mb)))?;
        Ok(())
    })
}

fn char_name(input: &str) -> IResult<&str, &str>
{
    let end = input.find(|c: char| !c.is_alpha() && ![' ', '\''].contains(&c));
    let (matched, rest) = if let Some(end) = end {
        input.split_at(end)
    } else {
        (input, "")
    };
    Ok((rest, matched))
}
