use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for fbtree_sys::FbU64 {
    const IGNORE_GET: bool = true;
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

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(self.iter(key).take(count));
    }
}

impl<H: index::Hasher> Index<String, H> for fbtree_sys::FbString {
    const IGNORE_GET: bool = true;
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<String, H> for &'_ fbtree_sys::FbString {
    type Handle<'a>
        = &'a fbtree_sys::FbString
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<String> for &'_ fbtree_sys::FbString {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        fbtree_sys::FbString::lookup(self, key)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        fbtree_sys::FbString::upsert(self, key, value);
        None
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        fbtree_sys::FbString::update(self, key, value);
        None
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(self.iter(key).take(count));
    }
}
