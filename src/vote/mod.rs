mod start;

use start::START_COMMAND;
use serenity::framework::standard::macros::group;

#[group]
#[commands(start)]
struct Vote;
