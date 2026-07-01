use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for contrie::CloneConMap<$map, u64> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                contrie::CloneConMap::default()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
            for &'_ contrie::CloneConMap<$map, u64>
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

impl index::IndexPin<u64, u64> for &'_ contrie::CloneConMap<u64, u64> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(contrie::CloneConMap::get(self, &key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(contrie::CloneConMap::get_or_insert(self, key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        core::hint::black_box(contrie::CloneConMap::insert(self, key, value));
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ contrie::CloneConMap<u128, u64> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(contrie::CloneConMap::get(self, &key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(contrie::CloneConMap::get_or_insert(self, key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        core::hint::black_box(contrie::CloneConMap::insert(self, key, value));
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl index::IndexPin<&'static [u8], u64> for &'_ contrie::CloneConMap<Box<[u8]>, u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(contrie::CloneConMap::get(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(contrie::CloneConMap::get_or_insert(
            self,
            Box::from(key),
            value,
        ));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(contrie::CloneConMap::insert(self, Box::from(key), value));
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl index::IndexPin<&'static [u8], u64> for &'_ contrie::CloneConMap<&'static [u8], u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(contrie::CloneConMap::get(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(contrie::CloneConMap::get_or_insert(self, key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(contrie::CloneConMap::insert(self, key, value));
    }
}
