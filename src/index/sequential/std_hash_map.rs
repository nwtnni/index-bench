use core::cell::UnsafeCell;

use crate::Index;
use crate::index;

pub struct HashMap<K, V, H>(UnsafeCell<std::collections::HashMap<K, V, H>>);

// Wildly unsafe
unsafe impl<K, V, H> Sync for HashMap<K, V, H> {}

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for HashMap<$map, u64, H> {
            const CONCURRENT: bool = false;

            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                HashMap(UnsafeCell::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ HashMap<$map, u64, H> {
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

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.get(&key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .or_insert(value),
        );
    }

    fn update(&mut self, key: u64, value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .and_modify(|old| *old = value),
        );
    }

    fn remove(&mut self, key: u64) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.remove(&key));
    }
}

impl_index!(u128, u128);

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ HashMap<u128, u64, H> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.get(&key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .or_insert(value),
        );
    }

    fn update(&mut self, key: u128, value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .and_modify(|old| *old = value),
        );
    }

    fn remove(&mut self, key: u128) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.remove(&key));
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64> for &'_ HashMap<Box<[u8]>, u64, H> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.get(key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(Box::from(key))
                .or_insert(value),
        );
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(Box::from(key))
                .and_modify(|old| *old = value),
        );
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.remove(key));
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64> for &'_ HashMap<&'static [u8], u64, H> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.get(key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .or_insert(value),
        );
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .entry(key)
                .and_modify(|old| *old = value),
        );
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() }.remove(key));
    }
}
