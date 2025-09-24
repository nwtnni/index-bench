use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for bztree::BzTree<K, u32> {
    type Handle<'a> = &'a Self;

    fn new() -> Self {
        bztree::BzTree::new()
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::Handle<K> for &'_ bztree::BzTree<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        bztree::BzTree::get(self, key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &crossbeam_epoch::pin();
        bztree::BzTree::upsert(self, key, value, guard).copied()
    }
}
