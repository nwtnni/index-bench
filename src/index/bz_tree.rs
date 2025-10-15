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

    fn range<'a>(&'a mut self, min: &'a K, max: &'a K, output: &mut Vec<(K, u32)>) {
        let guard = &crossbeam_epoch::pin();
        output.extend(
            bztree::BzTree::range(self, min..=max, guard).map(|(key, value)| (key.clone(), *value)),
        );
    }
}
