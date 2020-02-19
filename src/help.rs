use serenity::client::Context;
use serenity::framework::standard::help_commands::with_embeds;
use serenity::framework::standard::macros::help;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, HelpOptions};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;

#[help]
pub fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    with_embeds(context, msg, args, &help_options, groups, owners)
}
