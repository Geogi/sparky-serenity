use dotenv::dotenv;
use serenity::client::Client;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::{
    macros::{command, group, help},
    Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::prelude::{Context, EventHandler};
use std::collections::HashSet;
use std::env;

#[group]
#[commands(simple)]
struct Roleplay;

struct Handler;

impl EventHandler for Handler {}

#[help]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    with_embeds(context, msg, args, &help_options, groups, owners)
}

fn main() {
    dotenv().ok();

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("!")
                    .owners(vec![UserId(190183362294579211)].into_iter().collect())
            })
            .group(&ROLEPLAY_GROUP)
            .help(&MY_HELP),
    );

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn simple(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Ok(reply) = msg
        .channel_id
        .send_message(&ctx, |m| m.content("Quel jour ?"))
    {
        for em in &["🇱", "🇦", "🇪", "🇯", "🇻", "🇸", "🇩", "🚫"] {
            reply.react(&ctx, *em).ok();
        }
    }
    Ok(())
}
