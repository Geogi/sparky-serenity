use anyhow::{Context as _, Error};
use fehler::throws;
use reqwest::{
    blocking::{Client, Response},
    IntoUrl,
};
use serenity::client::Context;

struct HttpKey;
impl typemap::Key for HttpKey {
    type Value = Client;
}

#[throws]
pub fn get(ctx: &Context, url: impl IntoUrl) -> Response {
    let create = {
        let data_read = ctx.data.read();
        !data_read.contains::<HttpKey>()
    };
    if create {
        let mut data_write = ctx.data.write();
        data_write.insert::<HttpKey>(Client::new());
    }
    let data_read = ctx.data.read();
    let client = data_read
        .get::<HttpKey>()
        .context("http is not in context")?;
    client.get(url).send()?
}
