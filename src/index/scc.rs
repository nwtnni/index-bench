use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K>(Arc<scc::HashMap<K, u32, rapidhash::fast::RandomState>>);

impl<K: index::Key> Index<K> for Map<K> {
    type Handle = Self;

    fn new() -> Self {
        Self(Arc::new(scc::HashMap::with_hasher(
            rapidhash::fast::RandomState::new(),
        )))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        self.0.read_sync(key, |_, value| *value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        self.0.insert_sync(key, value).err().map(|(_, value)| value)
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
