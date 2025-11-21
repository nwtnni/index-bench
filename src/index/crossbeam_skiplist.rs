use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, H> for crossbeam_skiplist::SkipMap<$map, u64> {
            type Send<'a> = &'a Self;

            const IGNORE_INSERT: bool = true;
            const IGNORE_GET: bool = true;

            fn new(_: &index::Config) -> Self {
                crossbeam_skiplist::SkipMap::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, H>
            for &crossbeam_skiplist::SkipMap<$map, u64>
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

impl index::IndexPin<u64> for &'_ crossbeam_skiplist::SkipMap<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::get(self, &key)?.value())
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key, value).value())
    }

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            crossbeam_skiplist::SkipMap::range(self, key..)
                .take(count)
                .map(|entry| *entry.value()),
        )
    }
}

impl_index!(String, &'static str);

impl index::IndexPin<String> for &'_ crossbeam_skiplist::SkipMap<&'static str, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::get(self, &key)?.value())
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key, value).value())
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            crossbeam_skiplist::SkipMap::range(self, key..)
                .take(count)
                .map(|entry| *entry.value()),
        )
    }
}

impl_index!(String, String);

impl index::IndexPin<String> for &'_ crossbeam_skiplist::SkipMap<String, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::get(self, key)?.value())
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key.to_owned(), value).value())
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            crossbeam_skiplist::SkipMap::range::<str, _>(
                self,
                // NOTE: `key..` doesn't work due to 'static lifetime?
                // Not sure why (&*key).. doesn't work
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
            )
            .take(count)
            .map(|entry| *entry.value()),
        )
    }
}
