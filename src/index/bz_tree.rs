use core::ops::RangeBounds;

use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for bztree::BzTree<K, u32> {
    type Send<'a> = &'a Self;

    fn new() -> Self {
        bztree::BzTree::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ bztree::BzTree<K, u32> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ bztree::BzTree<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        bztree::BzTree::get(self, key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        bztree::BzTree::upsert(self, key, value, guard).copied()
    }

    fn range<'a, R: RangeBounds<&'a K>>(
        &'a mut self,
        range: R,
    ) -> impl Iterator<Item = (K, u32)> + 'a {
        let start = range.start_bound().map(|start| (**start).clone());
        let end = range.end_bound().map(|end| (**end).clone());
        let guard = &crossbeam_epoch::pin();
        bztree::BzTree::range(self, (start, end), guard)
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
