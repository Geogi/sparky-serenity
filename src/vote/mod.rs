mod start;

use serenity::framework::standard::macros::group;
use start::START_COMMAND;

#[group]
#[commands(start)]
struct Vote;
