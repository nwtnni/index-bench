use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for fbtree_sys::FbTree {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ fbtree_sys::FbTree {
    type Handle<'a>
        = &'a fbtree_sys::FbTree
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64> for &'_ fbtree_sys::FbTree {
    fn get(&mut self, _key: u64) -> Option<u64> {
        todo!()
        // fbtree_sys::FbTree::lookup(self, key)
    }

    fn insert(&mut self, _key: u64, _value: u64) -> Option<u64> {
        todo!()
        // fbtree_sys::FbTree::upsert(
        //     self,
        //     unsafe { core::mem::transmute_copy::<K, u64>(&key) },
        //     value,
        // );
        // None
    }

    fn update(&mut self, _key: u64, _value: u64) -> Option<u64> {
        todo!()
        // fbtree_sys::FbTree::update(
        //     self,
        //     unsafe { core::mem::transmute_copy::<K, u64>(&key) },
        //     value,
        // );
        // None
    }
}
