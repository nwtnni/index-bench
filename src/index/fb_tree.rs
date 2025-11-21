use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for fbtree_sys::FbU64 {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ fbtree_sys::FbU64 {
    type Handle<'a>
        = &'a fbtree_sys::FbU64
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64> for &'_ fbtree_sys::FbU64 {
    fn get(&mut self, key: u64) -> Option<u64> {
        fbtree_sys::FbU64::lookup(self, key)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        fbtree_sys::FbU64::upsert(self, key, value);
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        fbtree_sys::FbU64::update(self, key, value);
        None
    }
}
