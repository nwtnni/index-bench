use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K: index::Key>(Arc<bztree::BzTree<K, u32>>);

impl<K: index::Key, H: index::Hasher> Index<K, H> for Map<K> {
    type Handle = Self;

    fn new() -> Self {
        Self(Arc::new(bztree::BzTree::new()))
    }

    fn pin(&self) -> Self::Handle {
        Self(self.0.clone())
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        self.0.get(key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        self.0.upsert(key, value, guard).copied()
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
