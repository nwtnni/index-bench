use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, H> for dashmap::DashMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                dashmap::DashMap::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, H> for &'_ dashmap::DashMap<$map, u64, H> {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }
    };
}

impl_index!(u64, u64);

impl<H: index::Hasher> index::IndexPin<u64> for &'_ dashmap::DashMap<u64, u64, H> {
    fn get(&mut self, key: u64) -> Option<u64> {
        dashmap::DashMap::get(self, &key).map(|value| *value)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        dashmap::DashMap::remove(self, &key).map(|(_, value)| value)
    }
}

impl_index!(String, &'static str);

impl<H: index::Hasher> index::IndexPin<String> for &'_ dashmap::DashMap<&'static str, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        dashmap::DashMap::get(self, &key).map(|value| *value)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        dashmap::DashMap::remove(self, &key).map(|(_, value)| value)
    }
}

impl_index!(String, String);

impl<H: index::Hasher> index::IndexPin<String> for &'_ dashmap::DashMap<String, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        dashmap::DashMap::get(self, key).map(|value| *value)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key.to_owned(), value)
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        dashmap::DashMap::remove(self, key).map(|(_, value)| value)
    }
}
