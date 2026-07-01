use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for scc::HashIndex<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                scc::HashIndex::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H>
            for &'_ scc::HashIndex<$map, u64, H>
        {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }

        impl<H: index::Hasher> Index<$index, u64, H> for scc::HashMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                scc::HashMap::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ scc::HashMap<$map, u64, H> {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }

        impl<H: index::Hasher> Index<$index, u64, H> for scc::TreeIndex<$map, u64> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                scc::TreeIndex::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ scc::TreeIndex<$map, u64> {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                *self
            }
        }
    };
}

impl_index!(u64, u64);

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ scc::HashIndex<u64, u64, H> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(self.read_sync(&key, |_, value| *value));
    }

    fn insert(&mut self, key: u64, value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        if let scc::hash_index::Entry::Occupied(mut entry) = self.entry_sync(key) {
            entry.update(value);
        }
    }

    fn remove(&mut self, key: u64) {
        core::hint::black_box(self.remove_sync(&key));
    }
}

impl<H: index::Hasher> index::IndexPin<u64, u64> for &'_ scc::HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(self.read_sync(&key, |_, value| *value));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(self.upsert_sync(key, value));
    }

    fn update(&mut self, key: u64, value: u64) {
        core::hint::black_box(self.update_sync(&key, |_, old| {
            let save = *old;
            *old = value;
            save
        }));
    }

    fn remove(&mut self, key: u64) {
        core::hint::black_box(self.remove_sync(&key));
    }
}

impl index::IndexPin<u64, u64> for &'_ scc::TreeIndex<u64, u64> {
    fn get(&mut self, key: u64) {
        let guard = scc::Guard::new();
        core::hint::black_box(self.peek(&key, &guard));
    }

    fn insert(&mut self, key: u64, value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn remove(&mut self, key: u64) {
        core::hint::black_box(self.remove_sync(&key));
    }

    fn scan(&mut self, key: u64, count: usize) {
        let guard = scc::Guard::new();
        core::hint::black_box(
            scc::TreeIndex::range(self, key.., &guard)
                .take(count)
                .count(),
        );
    }
}

impl_index!(u128, u128);

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ scc::HashIndex<u128, u64, H> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(self.read_sync(&key, |_, value| *value));
    }

    fn insert(&mut self, key: u128, value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        if let scc::hash_index::Entry::Occupied(mut entry) = self.entry_sync(key) {
            entry.update(value);
        }
    }

    fn remove(&mut self, key: u128) {
        core::hint::black_box(self.remove_sync(&key));
    }
}

impl<H: index::Hasher> index::IndexPin<u128, u64> for &'_ scc::HashMap<u128, u64, H> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(self.read_sync(&key, |_, value| *value));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(self.upsert_sync(key, value));
    }

    fn update(&mut self, key: u128, value: u64) {
        core::hint::black_box(self.update_sync(&key, |_, old| {
            let save = *old;
            *old = value;
            save
        }));
    }

    fn remove(&mut self, key: u128) {
        core::hint::black_box(self.remove_sync(&key));
    }
}

impl index::IndexPin<u128, u64> for &'_ scc::TreeIndex<u128, u64> {
    fn get(&mut self, key: u128) {
        let guard = scc::Guard::new();
        core::hint::black_box(self.peek(&key, &guard));
    }

    fn insert(&mut self, key: u128, value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn remove(&mut self, key: u128) {
        core::hint::black_box(self.remove_sync(&key));
    }

    fn scan(&mut self, key: u128, count: usize) {
        let guard = scc::Guard::new();
        core::hint::black_box(
            scc::TreeIndex::range(self, key.., &guard)
                .take(count)
                .count(),
        );
    }
}

impl_index!(&'static [u8], Box<[u8]>);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ scc::HashIndex<Box<[u8]>, u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.read_sync(key, |_, value| *value));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(self.insert_sync(Box::from(key), value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        if let scc::hash_index::Entry::Occupied(mut entry) = self.entry_sync(Box::from(key)) {
            entry.update(value);
        }
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }
}

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64> for &'_ scc::HashMap<Box<[u8]>, u64, H> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.read_sync(key, |_, value| *value));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(self.upsert_sync(Box::from(key), value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(self.update_sync(key, |_, old| {
            let save = *old;
            *old = value;
            save
        }));
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }
}

impl index::IndexPin<&'static [u8], u64> for &'_ scc::TreeIndex<Box<[u8]>, u64> {
    fn get(&mut self, key: &'static [u8]) {
        let guard = scc::Guard::new();
        core::hint::black_box(self.peek(key, &guard));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(self.insert_sync(Box::from(key), value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        let guard = scc::Guard::new();
        core::hint::black_box(
            scc::TreeIndex::range::<[u8], _>(
                self,
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
                &guard,
            )
            .take(count)
            .count(),
        );
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ scc::HashIndex<&'static [u8], u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.read_sync(key, |_, value| *value));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        if let scc::hash_index::Entry::Occupied(mut entry) = self.entry_sync(key) {
            entry.update(value);
        }
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }
}

impl<H: index::Hasher> index::IndexPin<&'static [u8], u64>
    for &'_ scc::HashMap<&'static [u8], u64, H>
{
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.read_sync(key, |_, value| *value));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(self.upsert_sync(key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(self.update_sync(key, |_, old| {
            let save = *old;
            *old = value;
            save
        }));
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }
}

impl index::IndexPin<&'static [u8], u64> for &'_ scc::TreeIndex<&'static [u8], u64> {
    fn get(&mut self, key: &'static [u8]) {
        let guard = scc::Guard::new();
        core::hint::black_box(self.peek(key, &guard));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        let _ = core::hint::black_box(self.insert_sync(key, value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        core::hint::black_box(self.remove_sync(key));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        let guard = scc::Guard::new();
        core::hint::black_box(
            scc::TreeIndex::range::<[u8], _>(
                self,
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
                &guard,
            )
            .take(count)
            .count(),
        );
    }
}
