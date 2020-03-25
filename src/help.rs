use serenity::client::Context;
use serenity::framework::standard::help_commands::{with_embeds, create_customised_help_data, CustomisedHelpData, Command};
use serenity::framework::standard::macros::help;
use serenity::framework::standard::{Args, CommandGroup, CommandResult, HelpOptions};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;

#[help]
#[individual_command_tip = "Pour plus d’information sur une commande, donnez-la en argument à celle d’aide."]
#[suggestion_text = "La commande n’existe pas, pensiez-vous à `{}` ?"]
#[no_help_available_text = "Rien ne correspond dans l’aide."]
#[command_not_found_text = "La commande `{}` n’existe pas."]
#[lacking_role = "Hide"]
#[lacking_permissions = "Hide"]
#[lacking_ownership = "Hide"]
#[lacking_conditions = "Hide"]
#[wrong_channel = "Hide"]
#[max_levenshtein_distance(3)]
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
