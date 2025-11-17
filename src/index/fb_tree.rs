use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for fbtree_sys::FbTree {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        assert!(core::any::type_name::<K>() == "u64");
        const {
            assert!(core::mem::size_of::<usize>() == 8);
        }
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ fbtree_sys::FbTree {
    type Handle<'a>
        = &'a fbtree_sys::FbTree
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ fbtree_sys::FbTree {
    fn get(&mut self, key: &K) -> Option<u32> {
        fbtree_sys::FbTree::lookup(self, unsafe { core::mem::transmute_copy::<K, u64>(key) })
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        fbtree_sys::FbTree::upsert(
            self,
            unsafe { core::mem::transmute_copy::<K, u64>(&key) },
            value,
        );
        None
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        fbtree_sys::FbTree::update(
            self,
            unsafe { core::mem::transmute_copy::<K, u64>(&key) },
            value,
        );
        None
    }
}
