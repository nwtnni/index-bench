use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, u64, H> for bplustree::BPlusTree<K, u64> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        bplustree::BPlusTree::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, u64, H>
    for &'_ bplustree::BPlusTree<K, u64>
{
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K, u64> for &'_ bplustree::BPlusTree<K, u64> {
    fn get(&mut self, key: &K) -> Option<u64> {
        self.lookup(key, |value| *value)
    }

    fn insert(&mut self, key: K, value: u64) -> Option<u64> {
        bplustree::BPlusTree::insert(self, key, value)
    }
}
