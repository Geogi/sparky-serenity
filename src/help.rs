use crate::error::{ARes, AVoid};
use clap::{App, ArgMatches};
use serenity::{
    client::Context,
    framework::standard::help_commands::with_embeds,
    framework::standard::macros::help,
    framework::standard::{Args, CommandGroup, CommandResult, HelpOptions},
    model::channel::Message,
    model::id::UserId,
};
use std::collections::HashSet;

#[help]
#[individual_command_tip = "Pour plus d’information sur une commande, donnez-la en argument à \
celle d’aide.\nCertaines commandes (notées **ILC**) nécessitent d’être appelées avec `--help` \
comme argument pour davantage d’informations sur leur utilisation."]
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

pub fn clap_settings<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    use clap::AppSettings::*;
    app.settings(&[
        DontCollapseArgsInUsage,
        DisableVersion,
        HidePossibleValuesInHelp,
        NoBinaryName,
        UnifiedHelpMessage,
    ])
}

pub fn clap_help<'a>(
    ctx: &Context,
    msg: &Message,
    args: Args,
    app: App<'a, '_>,
) -> ARes<Option<ArgMatches<'a>>> {
    let help_app = app.clone();
    let is_err = match app.get_matches_from_safe(args.raw_quoted()) {
        Ok(args) => return Ok(Some(args)),
        Err(e) => e.kind != clap::ErrorKind::HelpDisplayed,
    };
    if is_err {
        clap_bad_use(ctx, msg, help_app.to_string())?;
    } else {
        let mut help = vec![];
        help_app.write_help(&mut help)?;
        let help = String::from_utf8(help)?;
        msg.channel_id
            .send_message(ctx, |m| m.embed(|e| e.title(help_app).description(help)))?;
    }
    Ok(None)
}

pub fn clap_bad_use(ctx: &Context, msg: &Message, name: String) -> AVoid {
    msg.reply(
        ctx,
        format!(
            "Utilisation incorrecte de la commande. Utilisez `{} --help` pour l’aide.",
            name
        ),
    )?;
    Ok(())
}
