use core::cell::UnsafeCell;
use core::ops::ControlFlow;

use crate::Index;
use crate::index;

pub struct Map<K: ::arctic::Key, V: ::arctic::concurrent::Value>(
    UnsafeCell<::arctic::sequential::Map<K, V>>,
);

// Wildly unsafe
unsafe impl<K: ::arctic::Key, V: ::arctic::concurrent::Value> Sync for Map<K, V> {}

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for Map<$map, u64> {
            const CONCURRENT: bool = false;

            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                Map(UnsafeCell::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ Map<$map, u64> {
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

impl index::IndexPin<u64, u64> for &'_ Map<u64, u64> {
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

    fn scan(&mut self, key: u64, mut count: usize) {
        let shard = unsafe { self.0.get().as_mut_unchecked() }.range(key..);

        shard.values::<arctic::Ascend>().for_each_internal(|_| {
            if count == 0 {
                ControlFlow::Break(())
            } else {
                count -= 1;
                ControlFlow::Continue(())
            }
        });
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ Map<u128, u64> {
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

    fn scan(&mut self, key: u128, mut count: usize) {
        let shard = unsafe { self.0.get().as_mut_unchecked() }.range(key..);

        shard.values::<arctic::Ascend>().for_each_internal(|_| {
            if count == 0 {
                ControlFlow::Break(())
            } else {
                count -= 1;
                ControlFlow::Continue(())
            }
        });
    }
}

impl_index!(
    &'static [u8],
    ::arctic::key::BoxedSlice<::arctic::key::Terminated<b'\n'>>
);

impl index::IndexPin<&'static [u8], u64>
    for &'_ Map<::arctic::key::BoxedSlice<::arctic::key::Terminated<b'\n'>>, u64>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .get(unsafe { ::arctic::key::Slice::new_unchecked(key) }),
        );
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .insert(unsafe { ::arctic::key::Slice::new_unchecked(key) }, value),
        );
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .update(unsafe { ::arctic::key::Slice::new_unchecked(key) }, value),
        );
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() })
            .remove(unsafe { ::arctic::key::Slice::new_unchecked(key) });
    }

    fn scan(&mut self, key: &'static [u8], mut count: usize) {
        let shard = unsafe { self.0.get().as_mut_unchecked() }.range(key..);

        shard.values::<arctic::Ascend>().for_each_internal(|_| {
            if count == 0 {
                ControlFlow::Break(())
            } else {
                count -= 1;
                ControlFlow::Continue(())
            }
        });
    }
}

impl_index!(
    &'static [u8],
    &'static ::arctic::key::Slice<::arctic::key::Terminated<b'\n'>>
);

impl index::IndexPin<&'static [u8], u64>
    for &'_ Map<&'static ::arctic::key::Slice<::arctic::key::Terminated<b'\n'>>, u64>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .get(unsafe { ::arctic::key::Slice::new_unchecked(key) }),
        );
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .insert(unsafe { ::arctic::key::Slice::new_unchecked(key) }, value),
        );
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(
            unsafe { self.0.get().as_mut_unchecked() }
                .update(unsafe { ::arctic::key::Slice::new_unchecked(key) }, value),
        );
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(unsafe { self.0.get().as_mut_unchecked() })
            .remove(unsafe { ::arctic::key::Slice::new_unchecked(key) });
    }

    fn scan(&mut self, key: &'static [u8], mut count: usize) {
        let shard = unsafe { self.0.get().as_mut_unchecked() }.range(key..);

        shard.values::<arctic::Ascend>().for_each_internal(|_| {
            if count == 0 {
                ControlFlow::Break(())
            } else {
                count -= 1;
                ControlFlow::Continue(())
            }
        });
    }
}
