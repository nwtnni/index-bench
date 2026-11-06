use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for bbtree::BBTreeMap<$map, u64> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                bbtree::BBTreeMap::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &bbtree::BBTreeMap<$map, u64> {
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

impl index::IndexPin<u64, u64> for &'_ bbtree::BBTreeMap<u64, u64> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(bbtree::BBTreeMap::get_copied(self, &key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        let _ = core::hint::black_box(bbtree::BBTreeMap::insert(self, key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        core::hint::black_box(bbtree::BBTreeMap::update(self, &key, |_, old| *old = value));
    }

    fn scan(&mut self, key: u64, mut count: usize) {
        core::hint::black_box(bbtree::BBTreeMap::scan(self, key.., |_, _| {
            if count == 0 {
                core::ops::ControlFlow::Break(())
            } else {
                count -= 1;
                core::ops::ControlFlow::Continue(())
            }
        }));
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ bbtree::BBTreeMap<u128, u64> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(bbtree::BBTreeMap::get_copied(self, &key));
    }

    fn insert(&mut self, key: u128, value: u64) {
        let _ = core::hint::black_box(bbtree::BBTreeMap::insert(self, key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        core::hint::black_box(bbtree::BBTreeMap::update(self, &key, |_, old| *old = value));
    }

    fn scan(&mut self, key: u128, mut count: usize) {
        core::hint::black_box(bbtree::BBTreeMap::scan(self, key.., |_, _| {
            if count == 0 {
                core::ops::ControlFlow::Break(())
            } else {
                count -= 1;
                core::ops::ControlFlow::Continue(())
            }
        }));
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl index::IndexPin<&'static [u8], u64> for &'_ bbtree::BBTreeMap<Box<[u8]>, u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(bbtree::BBTreeMap::get_copied(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(bbtree::BBTreeMap::insert(self, Box::from(key), value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(bbtree::BBTreeMap::update(self, key, |_, old| *old = value));
    }

    fn scan(&mut self, key: &'static [u8], mut count: usize) {
        core::hint::black_box(bbtree::BBTreeMap::scan::<[u8], _>(
            self,
            (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
            |_, _| {
                if count == 0 {
                    core::ops::ControlFlow::Break(())
                } else {
                    count -= 1;
                    core::ops::ControlFlow::Continue(())
                }
            },
        ));
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl index::IndexPin<&'static [u8], u64> for &'_ bbtree::BBTreeMap<&'static [u8], u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(bbtree::BBTreeMap::get_copied(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(bbtree::BBTreeMap::insert(self, key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(bbtree::BBTreeMap::update(self, &key, |_, old| *old = value));
    }

    fn scan(&mut self, key: &'static [u8], mut count: usize) {
        core::hint::black_box(bbtree::BBTreeMap::scan(self, key.., |_, _| {
            if count == 0 {
                core::ops::ControlFlow::Break(())
            } else {
                count -= 1;
                core::ops::ControlFlow::Continue(())
            }
        }));
    }
}
