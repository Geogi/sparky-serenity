use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

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
