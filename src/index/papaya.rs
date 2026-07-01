use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for papaya::HashMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                papaya::HashMap::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
            for &'_ papaya::HashMap<$map, u64, H>
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

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ papaya::HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) {
        let map = self.pin();
        core::hint::black_box(map.get(&key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        let map = self.pin();
        core::hint::black_box(map.insert(key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        let map = self.pin();
        core::hint::black_box(map.update(key, |_| value));
    }

    fn remove(&mut self, key: u64) {
        let map = self.pin();
        core::hint::black_box(map.remove(&key));
    }
}

impl_index!(u128, u128);

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ papaya::HashMap<u128, u64, H> {
    fn get(&mut self, key: u128) {
        let map = self.pin();
        core::hint::black_box(map.get(&key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        let map = self.pin();
        core::hint::black_box(map.insert(key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        let map = self.pin();
        core::hint::black_box(map.update(key, |_| value));
    }

    fn remove(&mut self, key: u128) {
        let map = self.pin();
        core::hint::black_box(map.remove(&key));
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ papaya::HashMap<Box<[u8]>, u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        let map = self.pin();
        core::hint::black_box(map.get(key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let map = self.pin();
        core::hint::black_box(map.insert(Box::from(key), value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        let map = self.pin();
        core::hint::black_box(map.update(Box::from(key), |_| value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        let map = self.pin();
        core::hint::black_box(map.remove(key));
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ papaya::HashMap<&'static [u8], u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        let map = self.pin();
        core::hint::black_box(map.get(key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let map = self.pin();
        core::hint::black_box(map.insert(key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        let map = self.pin();
        core::hint::black_box(map.update(key, |_| value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        let map = self.pin();
        core::hint::black_box(map.remove(key));
    }
}
