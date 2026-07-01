use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for crossbeam_skiplist::SkipMap<$map, u64> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                crossbeam_skiplist::SkipMap::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
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

impl index::IndexPin<u64, u64> for &'_ crossbeam_skiplist::SkipMap<u64, u64> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::get(self, &key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::insert(self, key, value));
    }

    fn scan(&mut self, key: u64, count: usize) {
        core::hint::black_box(
            crossbeam_skiplist::SkipMap::range(self, key..)
                .take(count)
                .count(),
        );
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ crossbeam_skiplist::SkipMap<u128, u64> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::get(self, &key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::insert(self, key, value));
    }

    fn scan(&mut self, key: u128, count: usize) {
        core::hint::black_box(
            crossbeam_skiplist::SkipMap::range(self, key..)
                .take(count)
                .count(),
        );
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl index::IndexPin<&'static [u8], u64> for &'_ crossbeam_skiplist::SkipMap<Box<[u8]>, u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::get(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::insert(
            self,
            Box::from(key),
            value,
        ));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        core::hint::black_box(
            crossbeam_skiplist::SkipMap::range::<[u8], _>(
                self,
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
            )
            .take(count)
            .count(),
        );
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl index::IndexPin<&'static [u8], u64> for &'_ crossbeam_skiplist::SkipMap<&'static [u8], u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::get(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(crossbeam_skiplist::SkipMap::insert(self, key, value));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        core::hint::black_box(
            crossbeam_skiplist::SkipMap::range::<[u8], _>(
                self,
                // NOTE: `key..` doesn't work due to 'static lifetime?
                // Not sure why (&*key).. doesn't work
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
            )
            .take(count)
            .count(),
        );
    }
}
