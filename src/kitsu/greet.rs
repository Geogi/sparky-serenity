use anyhow::Error;
use fehler::throws;
use serenity::{
    client::Context,
    model::{guild::Member, id::ChannelId},
    utils::{Colour, MessageBuilder},
};

const GENERAL_CHAN: ChannelId = ChannelId(587884188066250765);

#[throws]
pub fn send_embed(ctx: &Context, member: &Member) {
    GENERAL_CHAN.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Bienvenue à Kitsunebi !")
            .colour(Colour::ORANGE)
            .thumbnail("https://cdn.discordapp.com/icons/587883211225563160/\
    963cfb6277dbfbc14c25fb484801438d.webp")
    .description(
        MessageBuilder::new().push_italic_line("Kitsunebi (feu du renard): ")
    .push("Yōkai qui tire son nom des lanternes brillant dans la nuit dont la légende dit \
    qu'elles proviennent d'un soupir de renard.\n\n")
.push("Bienvenue ")
.mention(member)
.push(" sur le Discord de la CL des ")
.push_bold("Kitsunebi")
.push(" !\n\nMerci de lire attentivement notre règlement !"))
        })
    })?
}
