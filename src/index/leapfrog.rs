use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for leapfrog::LeapMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                // NOTE: 8 is default initial size
                leapfrog::LeapMap::with_capacity_and_hasher(8, H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
            for &'_ leapfrog::LeapMap<$map, u64, H>
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

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ leapfrog::LeapMap<u64, u64, H> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(leapfrog::LeapMap::get(self, &key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(leapfrog::LeapMap::try_insert(self, key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        core::hint::black_box(leapfrog::LeapMap::update(self, &key, value));
    }

    fn remove(&mut self, key: u64) {
        core::hint::black_box(leapfrog::LeapMap::remove(self, &key));
    }
}

impl_index!(u128, u128);

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ leapfrog::LeapMap<u128, u64, H> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(leapfrog::LeapMap::get(self, &key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(leapfrog::LeapMap::try_insert(self, key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        core::hint::black_box(leapfrog::LeapMap::update(self, &key, value));
    }

    fn remove(&mut self, key: u128) {
        core::hint::black_box(leapfrog::LeapMap::remove(self, &key));
    }
}

// impl_index!(&'static [u8], Box<[u8]>);
//
// impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
//     for &'_ leapfrog::LeapMap<Box<[u8]>, u64, H>
// {
//     fn get(&mut self, key: &'static [u8]) {
//         core::hint::black_box(leapfrog::LeapMap::get(self, key).map(|value| *value));
//     }
//
//     fn insert(&mut self, key: &'static [u8], value: u64) {
//         core::hint::black_box(leapfrog::LeapMap::insert(self, Box::from(key), value));
//     }
//
//     fn remove(&mut self, key: &'static [u8]) {
//         core::hint::black_box(leapfrog::LeapMap::remove(self, key).map(|(_, value)| value));
//     }
// }

impl_index!(&'static [u8], &'static [u8]);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ leapfrog::LeapMap<&'static [u8], u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(leapfrog::LeapMap::get(self, &key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(leapfrog::LeapMap::insert(self, key, value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(leapfrog::LeapMap::remove(self, &key));
    }
}
