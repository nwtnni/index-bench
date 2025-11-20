use core::cell::Cell;

use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for papaya::HashMap<u64, u64, H> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        papaya::HashMap::with_hasher(H::default())
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ papaya::HashMap<u64, u64, H> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexPin<u64> for &'_ papaya::HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) -> Option<u64> {
        let map = self.pin();
        map.get(&key).copied()
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        let map = self.pin();
        map.insert(key, value).copied()
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        let map = self.pin();
        map.update(key, |_| value).copied()
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        let map = self.pin();
        map.remove(&key).copied()
    }

    fn increment(&mut self, key: u64) -> Option<u64> {
        let map = self.pin();
        let old = Cell::new(None);
        map.update_or_insert(
            key,
            |count| {
                old.set(Some(*count));
                count + 1
            },
            1,
        );
        old.into_inner()
    }
}
