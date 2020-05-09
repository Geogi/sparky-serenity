use crate::error::ARes;
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

pub fn find_message_with<T>(
    ctx: &Context,
    base: &Message,
    pred: impl FnMut(&Message) -> Option<T>,
) -> ARes<(Message, T)> {
    find_message_with_limit(ctx, base, pred, FIND_MESSAGE_LIMIT)
}

pub fn find_message_with_limit<T>(
    ctx: &Context,
    base: &Message,
    mut pred: impl FnMut(&Message) -> Option<T>,
    limit: usize,
) -> ARes<(Message, T)> {
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
            if counter > limit {
                bail!("message not found: limit reached");
            }
            if let Some(v) = pred(&msg) {
                return Ok((msg, v));
            }
            current = msg.id;
        }
        first = current;
    }
}

pub fn clap_name<'a, S: Into<&'a str>>(name: S) -> String {
    format!("{}{}", crate::PREFIX, name.into())
}
