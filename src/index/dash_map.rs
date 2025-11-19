use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for dashmap::DashMap<K, u32, H> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        dashmap::DashMap::with_hasher(H::default())
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ dashmap::DashMap<K, u32, H> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexPin<K> for &'_ dashmap::DashMap<K, u32, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        dashmap::DashMap::get(self, key).map(|value| *value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        dashmap::DashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        dashmap::DashMap::remove(self, &key).map(|(_, value)| value)
    }

    fn increment(&mut self, key: K) -> Option<u32> {
        let mut old = Some(0);
        let mut entry = dashmap::DashMap::entry(self, key).or_insert_with(|| {
            old = None;
            0
        });
        if old.is_some() {
            old = Some(*entry);
        }
        *entry += 1;
        old
    }
}
