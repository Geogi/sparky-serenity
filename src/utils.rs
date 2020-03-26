use crate::error::ARes;
use crate::state::{extract, Embedded};
use anyhow::bail;
use serenity::client::Context;
use serenity::model::channel::Message;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

const FIND_MESSAGE_LIMIT: usize = 1000;

pub trait MapExt<K, V> {
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq;
    fn insert(&mut self, k: K, v: V) -> Option<V>;
    fn modify<Q: ?Sized>(&mut self, k: K, f: fn(V) -> V)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.remove(k.borrow()).map(f).map(|v| self.insert(k, v));
    }
}

impl<K: Hash + Eq, V> MapExt<K, V> for HashMap<K, V> {
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        HashMap::remove(self, k)
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        HashMap::insert(self, k, v)
    }
}

pub fn find_message(
    ctx: &Context,
    base: &Message,
    mut pred: impl FnMut(&Embedded) -> bool,
) -> ARes<Message> {
    let mut counter = 0;
    let mut first = base.id;
    let mut current = first;
    loop {
        let messages = base.channel_id.messages(ctx, |r| r.before(first))?;
        if messages.is_empty() {
            bail!("message not found: no more messages");
        }
        for msg in messages {
            counter += 1;
            if counter > FIND_MESSAGE_LIMIT {
                bail!("message not found: limit reached");
            }
            if let Some(data) = extract(ctx, &msg) {
                if pred(&data) {
                    return Ok(msg);
                }
            }
            current = msg.id;
        }
        first = current;
    }
}

pub fn clap_name<'a, S: Into<&'a str>>(name: S) -> String {
    format!("{}{}", crate::PREFIX, name.into())
}
