#![allow(dead_code)]
#![allow(unused_variables)]
use crate::error::{wrap_cmd_err, ARes};
use crate::help::{clap_bad_use, clap_help, clap_settings};
use crate::utils::clap_name;
use anyhow::anyhow;
use chrono::Duration;
use clap::{App, Arg};
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::{map_res, opt};
use nom::multi::{fold_many1, separated_nonempty_list};
use nom::sequence::delimited;
use nom::IResult;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::utils::MessageBuilder;
use std::cmp::min;
use std::num::ParseIntError;
use std::str::FromStr;

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
                (1tc, 1m,\n10m, 30m, 1h, 1j, 1s, 1M). Seuls `R` et `S,` (avec la virgule) \
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
        let test = if let Ok(test) = parse_test(
            args.value_of("EXPR")
                .ok_or_else(|| anyhow!("unreachable: no expr"))?,
        ) {
            test
        } else {
            clap_bad_use(ctx, msg, app_name)?;
            return Ok(());
        };
        let mut rng = thread_rng();
        let (summary, details) = resolve(&mut rng, test.1);
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
    threshold: Option<u64>,
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
                if let Some(threshold) = threshold {
                    if res.hits < threshold {
                        "Échec".to_owned()
                    } else if res.hits == threshold {
                        "Réussite de justesse ou frôlé".to_owned()
                    } else {
                        format!(
                            "Réussite avec {} succès excédentaires",
                            res.hits - threshold
                        )
                    }
                } else {
                    format!("{} réussites", res.hits)
                }
            };
            let mut details = String::from("Détail du jet : ");
            details.push_str(&res.fmt_details());
            (summary, details)
        }
        ShadowrunTest::Extended(_) => unimplemented!(),
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

fn parse_test(input: &str) -> IResult<&str, ShadowrunTest> {
    let (input, pool) = series(input)?;
    let (input, edge) = opt(char('*'))(input)?;
    let (input, limit) = opt(delimited(char('['), series, char(']')))(input)?;
    let (input, threshold) = opt(delimited(char('('), series, char(')')))(input)?;
    Ok((
        input,
        ShadowrunTest::Simple(ShadowrunSimple {
            pool,
            edge: edge.is_some(),
            limit,
            threshold,
        }),
    ))
}

fn series(input: &str) -> IResult<&str, u64> {
    map_res(separated_nonempty_list(tag("+"), digit1), sum_series)(input)
}

fn sum_series(input: Vec<&str>) -> Result<u64, ParseIntError> {
    input.iter().fold(Ok(0), |a, i| Ok(a? + parse_int(i)?))
}

fn parse_int(input: &str) -> Result<u64, ParseIntError> {
    Ok(u64::from_str(input)?)
}
