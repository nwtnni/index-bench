use crate::index;
use crate::Index;

impl<H: index::Hasher> Index<u64, H> for hot_sys::HotTreeU64 {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ hot_sys::HotTreeU64 {
    type Handle<'a> = &'a hot_sys::HotTreeU64
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64> for &'_ hot_sys::HotTreeU64 {
    fn get(&mut self, key: u64) -> Option<u64> {
        let mut value = 0u64;
        if hot_sys::HotTreeU64::search_u64(self, key, &mut value) {
            Some(value)
        } else {
            None
        }
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        hot_sys::HotTreeU64::upsert_u64(self, key, value);
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        // Semantically same as insert for this index.
        hot_sys::HotTreeU64::upsert_u64(self, key, value);
        None
    }
}

impl<H: index::Hasher> Index<String, H> for hot_sys::HotTreeString {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<String, H> for &'_ hot_sys::HotTreeString {
    type Handle<'a> = &'a hot_sys::HotTreeString
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<String> for &'_ hot_sys::HotTreeString {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        hot_sys::HotTreeString::search(self, key)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        hot_sys::HotTreeString::upsert(self, key, value);
        None
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        hot_sys::HotTreeString::upsert(self, key, value);
        None
    }
}
