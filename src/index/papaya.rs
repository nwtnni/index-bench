use core::cell::Cell;

use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for papaya::HashMap<K, u32, H> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        papaya::HashMap::with_hasher(H::default())
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ papaya::HashMap<K, u32, H> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexPin<K> for &'_ papaya::HashMap<K, u32, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let map = self.pin();
        map.get(key).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let map = self.pin();
        map.insert(key, value).copied()
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        let map = self.pin();
        map.update(key, |_| value).copied()
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        let map = self.pin();
        map.remove(&key).copied()
    }

    fn increment(&mut self, key: K) -> Option<u32> {
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
