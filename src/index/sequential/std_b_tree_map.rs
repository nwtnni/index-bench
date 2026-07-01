use core::cell::UnsafeCell;

use crate::Index;
use crate::index;

pub struct BTreeMap<K, V>(UnsafeCell<std::collections::BTreeMap<K, V>>);

// Wildly unsafe
unsafe impl<K, V> Sync for BTreeMap<K, V> {}

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for BTreeMap<$map, u64> {
            const CONCURRENT: bool = false;

            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                BTreeMap(UnsafeCell::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ BTreeMap<$map, u64> {
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

impl index::IndexPin<u64, u64> for &'_ BTreeMap<u64, u64> {
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

    fn scan(&mut self, key: u64, count: usize) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .range(key..)
                .take(count)
                .count(),
        );
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ BTreeMap<u128, u64> {
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

    fn scan(&mut self, key: u128, count: usize) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .range(key..)
                .take(count)
                .count(),
        );
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl index::IndexPin<&'static [u8], u64> for &'_ BTreeMap<Box<[u8]>, u64> {
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

    fn scan(&mut self, key: &'static [u8], count: usize) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .range::<[u8], _>((core::ops::Bound::Included(key), core::ops::Bound::Unbounded))
                .take(count)
                .count(),
        );
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl index::IndexPin<&'static [u8], u64> for &'_ BTreeMap<&'static [u8], u64> {
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

    fn scan(&mut self, key: &'static [u8], count: usize) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .range::<[u8], _>((core::ops::Bound::Included(key), core::ops::Bound::Unbounded))
                .take(count)
                .count(),
        );
    }
}
