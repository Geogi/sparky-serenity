use serenity::{client::Context};
use reqwest::{IntoUrl, blocking::{Response, Client}};
use fehler::throws;
use anyhow::{Context as _, Error};

struct HttpKey;
impl typemap::Key for HttpKey {
    type Value = Client;
}

#[throws]
pub fn get(ctx: &Context, url: impl IntoUrl) -> Response {
    let create = {
        let data_read = ctx.data.read();
        data_read.get::<HttpKey>().is_none()
    };
    if create {
        let mut data_write = ctx.data.write();
        data_write.insert::<HttpKey>(Client::new());
    }
    let data_read = ctx.data.read();
    let client = data_read.get::<HttpKey>().context("http is not in context")?;
    client.get(url).send()?
}