// TODO: work in progress
#![allow(dead_code, unused_variables)]

use crate::{
    error::wrap_cmd_err,
    help::{clap_bad_use, clap_help, clap_settings},
    utils::clap_name,
};
use anyhow::anyhow;
use chrono::Duration;
use clap::{App, Arg};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, map_res, opt},
    multi::fold_many0,
    sequence::{delimited, tuple},
    IResult,
};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, Rng,
};
use serenity::{
    client::Context,
    framework::standard::macros::command,
    framework::standard::{Args, CommandResult},
    model::channel::Message,
    utils::MessageBuilder,
};
use std::{
    cmp::{min, Ordering},
    num::ParseIntError,
    str::FromStr,
};

#[command]
#[description = "Lance des dés.\n***ILC :** appelez avec `--help` \
pour l’utilisation.*"]
pub fn roll(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    wrap_cmd_err(|| {
        let app_name = clap_name("sr roll");
        let app = App::new(app_name.clone())
            .about("Lance des dés.")
            .long_about(
                "**Lance des dés**\n\
                Par défaut en mode Shadowrun. D’autres sont accessibles par des options.\n\
                En mode Shadowrun, trois formes sont supportées :\n\
                • `R*[L](S)` : **Jet simple.** `R` réserve, `*` chance utilisée, `L` limite, `S` \
                seuil. Seul `R` obligatoire.\n\
                • `R*C[L](S,T)` : **Jet étendu.** `C` nombre de chances utilisées, `T` intervalle \
                (1tc, 1m,\n10m, 30m, 1h, 1j, 1S, 1M). Seuls `R` et `S,` (avec la virgule) \
                obligatoires.\n\
                • `R1*[L1]-R2*[L2]` : **Jet opposé.** Seuls `R1` et `-R2` obligatoires.\n\
                Toutes les valeurs sauf `T` peuvent être une série de termes.",
            )
            .arg(Arg::with_name("EXPR").help(
                "Expression de lancer de dés. Cf. description (mode Shadowrun, par défaut)\n\
                ou options (modes alternatifs).",
            ));
        let app = clap_settings(app);
        let args = match clap_help(ctx, msg, args, app)? {
            Some(args) => args,
            None => return Ok(()),
        };
        let test = if let Ok(("", test)) = shadowrun_tests(
            args.value_of("EXPR")
                .ok_or_else(|| anyhow!("unreachable: no expr"))?,
        ) {
            test
        } else {
            clap_bad_use(ctx, msg, app_name)?;
            return Ok(());
        };
        let mut rng = thread_rng();
        let (summary, details) = resolve(&mut rng, test);
        msg.channel_id
            .send_message(ctx, |m| m.embed(|e| e.title(summary).description(details)))?;
        Ok(())
    })
}

enum ShadowrunTest {
    Simple(ShadowrunSimple),
    Extended(ShadowrunExtended),
    Opposed(ShadowrunOpposed),
}

struct ShadowrunSimple {
    pool: u64,
    edge: bool,
    limit: Option<u64>,
    threshold: Option<u64>,
}

struct ShadowrunExtended {
    pool: u64,
    edge_number: u64,
    limit: Option<u64>,
    threshold: u64,
    interval: Option<Duration>,
}

struct ShadowrunOpposed {
    my_pool: u64,
    my_edge: bool,
    my_limit: Option<u64>,
    their_pool: u64,
    their_edge: bool,
    their_limit: Option<u64>,
}

#[derive(Clone)]
struct ShadowrunRoll {
    hits: u64,
    glitch: bool,
    all: Vec<u8>,
}

impl ShadowrunRoll {
    fn fmt_details(&self) -> String {
        let mut mb = MessageBuilder::new();
        let mut all_peek = self.all.iter().peekable();
        while let Some(single) = all_peek.next() {
            if single == &1 {
                mb.push_mono(single);
            } else if single >= &5 {
                mb.push_bold(single);
            } else {
                mb.push(single);
            }
            if all_peek.peek().is_some() {
                mb.push(", ");
            }
        }
        mb.0
    }

    fn fumble(&self) -> bool {
        self.hits == 0 && self.glitch
    }
}

enum ExtendedOutcome {
    Success,
    Fumble,
    GlitchedBelowOne,
    PoolExhausted,
}

fn resolve(rng: &mut impl Rng, test: ShadowrunTest) -> (String, String) {
    match test {
        ShadowrunTest::Simple(ShadowrunSimple {
            pool,
            edge,
            limit,
            threshold,
        }) => {
            let res = shadowrun_roll(rng, pool, edge, limit);
            let summary = if res.fumble() {
                "Échec critique !".to_owned()
            } else {
                let glitch = if res.glitch {
                    " – complication !"
                } else {
                    "."
                };
                if let Some(threshold) = threshold {
                    match res.hits.cmp(&threshold) {
                        Ordering::Less => format!("Échec{}", glitch),
                        Ordering::Equal => format!("Réussite de justesse ou frôlé{}", glitch),
                        Ordering::Greater => format!(
                            "Réussite avec {} succès excédentaires{}",
                            res.hits - threshold,
                            glitch
                        ),
                    }
                } else {
                    format!(
                        "{} réussite{}{}",
                        res.hits,
                        if res.hits > 1 { "s" } else { "" },
                        glitch
                    )
                }
            };
            let mut details = format!(
                "Détail du jet {}: ",
                if edge { "(avec chance) " } else { "" }
            );
            details.push_str(&res.fmt_details());
            (summary, details)
        }
        ShadowrunTest::Extended(ShadowrunExtended {
            mut pool,
            mut edge_number,
            limit,
            threshold,
            interval,
        }) => {
            let mut all_res = vec![];
            let mut hits: u64 = 0;
            let mut penalties = vec![];
            let result = loop {
                if pool == 0 {
                    break ExtendedOutcome::PoolExhausted;
                }
                let partial = shadowrun_roll(rng, pool, edge_number > 0, limit);
                all_res.push(partial.clone());
                if partial.fumble() {
                    break ExtendedOutcome::Fumble;
                }
                if partial.glitch {
                    let penalty = Uniform::new_inclusive(1, 6).sample(rng);
                    penalties.push(penalty);
                    hits = hits.saturating_sub(penalty);
                    if hits == 0 {
                        break ExtendedOutcome::GlitchedBelowOne;
                    }
                }
                hits += partial.hits;
                if hits >= threshold {
                    break ExtendedOutcome::Success;
                }
                pool = pool.saturating_sub(1);
                edge_number = edge_number.saturating_sub(1);
            };
            let mut details = String::new();
            let mut nr = 0;
            for (i, res) in all_res.iter().enumerate() {
                nr += res.hits;
                details.push_str(&format!(
                    "Lancer {} : {}{}\n",
                    i + 1,
                    res.fmt_details(),
                    if res.fumble() {
                        " | Échec critique ! Le test s’arrête ici...".to_owned()
                    } else if res.glitch {
                        let pen = penalties.pop().unwrap();
                        nr = nr.saturating_sub(pen);
                        format!(
                            " | Complication ! Pénalité de {} réussites, plus que {}.",
                            pen, nr
                        )
                    } else {
                        format!(" | {} réussites.", nr)
                    }
                ))
            }
            let summary = match result {
                ExtendedOutcome::Success => {
                    format!("Réussite au bout de {} intervalles", all_res.len())
                }
                ExtendedOutcome::Fumble => "Échec critique !".to_string(),
                ExtendedOutcome::GlitchedBelowOne => "Échec à force de complications !".to_string(),
                ExtendedOutcome::PoolExhausted => "Échec faute de réserve !".to_string(),
            };
            (summary, details)
        }
        ShadowrunTest::Opposed(_) => unimplemented!(),
    }
}

fn shadowrun_roll(
    rng: &mut impl Rng,
    mut pool: u64,
    edge: bool,
    mut limit: Option<u64>,
) -> ShadowrunRoll {
    let mut hits = 0;
    let mut unrolled = pool;
    let mut ones = 0;
    let mut all = vec![];
    if edge {
        limit = None;
    }
    while unrolled > 0 {
        unrolled -= 1;
        let single = Uniform::new_inclusive(1, 6).sample(rng);
        all.push(single);
        if single >= 5 {
            hits += 1;
        }
        if single == 1 {
            ones += 1;
        }
        if edge && single == 6 {
            pool += 1;
            unrolled += 1;
        }
    }
    if let Some(limit) = limit {
        hits = min(hits, limit);
    }
    let glitch = ones >= pool / 2 + pool % 2;
    ShadowrunRoll { hits, glitch, all }
}

fn shadowrun_tests(input: &str) -> IResult<&str, ShadowrunTest> {
    alt((
        map(opposed_test, ShadowrunTest::Opposed),
        map(extended_test, ShadowrunTest::Extended),
        map(simple_test, ShadowrunTest::Simple),
    ))(input)
}

fn opposed_test(input: &str) -> IResult<&str, ShadowrunOpposed> {
    map(tag("lolilol"), |i| unimplemented!())(input)
}

fn intervals(input: &str) -> IResult<&str, Duration> {
    let (input, vstr) = alt((
        tag("1tc"),
        tag("1m"),
        tag("10m"),
        tag("30m"),
        tag("1h"),
        tag("1j"),
        tag("1S"),
        tag("1M"),
    ))(input)?;
    Ok((
        input,
        match vstr {
            "1tc" => Duration::seconds(3),
            "1m" => Duration::minutes(1),
            "10m" => Duration::minutes(10),
            "30m" => Duration::minutes(30),
            "1h" => Duration::hours(1),
            "1j" => Duration::days(1),
            "1S" => Duration::weeks(1),
            "1M" => Duration::days(30),
            _ => unreachable!("previously matched"),
        },
    ))
}

fn extended_test(input: &str) -> IResult<&str, ShadowrunExtended> {
    let (input, pool) = series(input)?;
    let (input, edge) = opt(char('*'))(input)?;
    let (input, edge_count) = opt(series)(input)?;
    let (input, limit) = opt(delimited(char('['), series, char(']')))(input)?;
    let (input, threshold) = delimited(char('('), series, char(','))(input)?;
    let (input, interval) = opt(intervals)(input)?;
    let (input, _) = char(')')(input)?;
    Ok((
        input,
        ShadowrunExtended {
            pool,
            edge_number: edge_count.unwrap_or(if edge.is_some() { 1 } else { 0 }),
            limit,
            threshold,
            interval,
        },
    ))
}

fn simple_test(input: &str) -> IResult<&str, ShadowrunSimple> {
    let (input, pool) = series(input)?;
    let (input, edge) = opt(char('*'))(input)?;
    let (input, limit) = opt(delimited(char('['), series, char(']')))(input)?;
    let (input, threshold) = opt(delimited(char('('), series, char(')')))(input)?;
    Ok((
        input,
        ShadowrunSimple {
            pool,
            edge: edge.is_some(),
            limit,
            threshold,
        },
    ))
}

fn series(input: &str) -> IResult<&str, u64> {
    let (input, head) = map_res(digit1, parse_int)(input)?;
    fold_many0(
        tuple((alt((char('+'), char('-'))), map_res(digit1, parse_int))),
        head,
        series_sum,
    )(input)
}

fn series_sum(acc: u64, item: (char, u64)) -> u64 {
    match item.0 {
        '+' => acc.saturating_add(item.1),
        '-' => acc.saturating_sub(item.1),
        _ => unreachable!("should not parse"),
    }
}

fn parse_int(input: &str) -> Result<u64, ParseIntError> {
    Ok(u64::from_str(input)?)
}
