use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K>(Arc<papaya::HashMap<K, u32, rapidhash::fast::RandomState>>);

impl<K: index::Key> Index<K> for Map<K> {
    type Handle = Self;

    fn new() -> Self {
        Self(Arc::new(papaya::HashMap::with_hasher(
            rapidhash::fast::RandomState::new(),
        )))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let map = self.0.pin();
        map.get(key).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let map = self.0.pin();
        map.insert(key, value).copied()
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
