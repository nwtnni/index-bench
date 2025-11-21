use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, H> for concurrent_map::ConcurrentMap<$map, u64> {
            type Send<'a> = Self;

            fn new(_: &index::Config) -> Self {
                concurrent_map::ConcurrentMap::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self.clone()
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, H>
            for concurrent_map::ConcurrentMap<$map, u64>
        {
            type Handle<'a> = &'a Self;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }
    };
}

impl_index!(u64, u64);

impl index::IndexPin<u64> for &'_ concurrent_map::ConcurrentMap<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::get(self, &key)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            concurrent_map::ConcurrentMap::range(self, key..)
                .take(count)
                .map(|(_, value)| value),
        );
    }
}

impl_index!(String, &'static str);

impl index::IndexPin<String> for &'_ concurrent_map::ConcurrentMap<&'static str, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        concurrent_map::ConcurrentMap::get(self, &key)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            concurrent_map::ConcurrentMap::range(self, key..)
                .take(count)
                .map(|(_, value)| value),
        );
    }
}

impl_index!(String, String);

impl index::IndexPin<String> for &'_ concurrent_map::ConcurrentMap<String, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        concurrent_map::ConcurrentMap::get(self, key)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::insert(self, key.to_owned(), value)
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            concurrent_map::ConcurrentMap::range::<str, _>(
                self,
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
            )
            .take(count)
            .map(|(_, value)| value),
        );
    }
}
