use crate::Index;
use crate::index;
use hot_sys::HotTreeString;
use hot_sys::HotTreeU64;

impl<H: index::Hasher> Index<u64, u64, H> for HotTreeU64 {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, u64, H> for &'_ HotTreeU64 {
    type Handle<'a>
        = &'a HotTreeU64
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64, u64> for &'_ HotTreeU64 {
    fn get(&mut self, key: u64) -> Option<u64> {
        let mut value = 0u64;
        if HotTreeU64::search_u64(self, key, &mut value) {
            Some(value)
        } else {
            None
        }
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        HotTreeU64::upsert_u64(self, key, value);
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        // Semantically same as insert for this index.
        HotTreeU64::upsert_u64(self, key, value);
        None
    }
}

impl<H: index::Hasher> Index<u128, u64, H> for HotTreeString {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u128, u64, H> for &'_ HotTreeString {
    type Handle<'a>
        = &'a HotTreeString
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u128, u64> for &'_ HotTreeString {
    fn get(&mut self, key: u128) -> Option<u64> {
        let key = key.to_be_bytes();
        HotTreeString::search(self, &key)
    }

    fn insert(&mut self, key: u128, value: u64) -> Option<u64> {
        let key = key.to_be_bytes();
        HotTreeString::upsert(self, &key, value);
        None
    }

    fn update(&mut self, key: u128, value: u64) -> Option<u64> {
        let key = key.to_be_bytes();
        HotTreeString::upsert(self, &key, value);
        None
    }
}

impl<H: index::Hasher> Index<Vec<u8>, u64, H> for HotTreeString {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<Vec<u8>, u64, H> for &'_ HotTreeString {
    type Handle<'a>
        = &'a HotTreeString
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<Vec<u8>, u64> for &'_ HotTreeString {
    fn get(&mut self, key: &'static [u8]) -> Option<u64> {
        HotTreeString::search(self, key)
    }

    fn insert(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        HotTreeString::upsert(self, key, value);
        None
    }

    fn update(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        HotTreeString::upsert(self, key, value);
        None
    }
}
