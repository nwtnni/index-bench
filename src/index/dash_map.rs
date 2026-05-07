use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for dashmap::DashMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                dashmap::DashMap::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
            for &'_ dashmap::DashMap<$map, u64, H>
        {
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

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ dashmap::DashMap<u64, u64, H> {
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

impl_index!(u128, u128);

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ dashmap::DashMap<u128, u64, H> {
    fn get(&mut self, key: u128) -> Option<u64> {
        dashmap::DashMap::get(self, &key).map(|value| *value)
    }

    fn insert(&mut self, key: u128, value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: u128) -> Option<u64> {
        dashmap::DashMap::remove(self, &key).map(|(_, value)| value)
    }
}

impl_index!(Vec<u8>, &'static [u8]);

impl<H: index::Hasher> index::IndexPin<Vec<u8>, u64>
    for &'_ dashmap::DashMap<&'static [u8], u64, H>
{
    fn get(&mut self, key: &'static [u8]) -> Option<u64> {
        dashmap::DashMap::get(self, &key).map(|value| *value)
    }

    fn insert(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key, value)
    }

    fn remove(&mut self, key: &'static [u8]) -> Option<u64> {
        dashmap::DashMap::remove(self, &key).map(|(_, value)| value)
    }
}

impl_index!(Vec<u8>, Vec<u8>);

impl<H: index::Hasher> index::IndexPin<Vec<u8>, u64> for &'_ dashmap::DashMap<Vec<u8>, u64, H> {
    fn get(&mut self, key: &'static [u8]) -> Option<u64> {
        dashmap::DashMap::get(self, key).map(|value| *value)
    }

    fn insert(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        dashmap::DashMap::insert(self, key.to_owned(), value)
    }

    fn remove(&mut self, key: &'static [u8]) -> Option<u64> {
        dashmap::DashMap::remove(self, key).map(|(_, value)| value)
    }
}
