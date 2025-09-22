use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K: index::Key>(Arc<crossbeam_skiplist::SkipMap<K, u32>>);

impl<K: index::Key> Index<K> for Map<K> {
    type Handle = Self;
    const INSERT_OLD: bool = false;

    fn new() -> Self {
        Self(Arc::new(crossbeam_skiplist::SkipMap::new()))
    }

    fn pin(&self) -> Self::Handle {
        Self(self.0.clone())
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        Some(*self.0.get(key)?.value())
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        Some(*self.0.insert(key, value).value())
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
