use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K, H: index::Hasher>(Arc<dashmap::DashMap<K, u32, H>>);

impl<K: index::Key, H: index::Hasher> Index<K, H> for Map<K, H> {
    type Handle = Self;

    fn new() -> Self {
        Self(Arc::new(dashmap::DashMap::with_hasher(H::default())))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }
}

impl<K: index::Key, H: index::Hasher> index::Handle<K> for Map<K, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        self.0.get(key).map(|value| *value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        self.0.insert(key, value)
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
