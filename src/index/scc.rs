use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, H> for scc::HashMap<$map, u64, H> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                scc::HashMap::with_hasher(H::default())
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, H> for &'_ scc::HashMap<$map, u64, H> {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }

        impl<H: index::Hasher> Index<$index, H> for scc::TreeIndex<$map, u64> {
            type Send<'a> = &'a Self;

            const IGNORE_INSERT: bool = true;

            fn new(_: &index::Config) -> Self {
                scc::TreeIndex::new()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, H> for &'_ scc::TreeIndex<$map, u64> {
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

impl<H: index::Hasher> index::IndexPin<u64> for &'_ scc::HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) -> Option<u64> {
        self.read_sync(&key, |_, value| *value)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        self.upsert_sync(key, value)
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        self.update_sync(&key, |_, old| {
            let save = *old;
            *old = value;
            save
        })
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        self.remove_sync(&key).map(|(_, value)| value)
    }
}

impl index::IndexPin<u64> for &'_ scc::TreeIndex<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        let guard = scc::Guard::new();
        self.peek(&key, &guard).copied()
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        // NOTE: does not insert if exists
        self.insert_sync(key, value).err().map(|(_, value)| value)
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        self.remove_sync(&key).then_some(0)
    }

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        let guard = scc::Guard::new();
        buffer.extend(
            scc::TreeIndex::range(self, key.., &guard)
                .take(count)
                .map(|(_, value)| *value),
        );
    }
}

impl_index!(String, &'static str);

impl<H: index::Hasher> index::IndexPin<String> for &'_ scc::HashMap<&'static str, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        self.read_sync(&key, |_, value| *value)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        self.upsert_sync(key, value)
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        self.update_sync(&key, |_, old| {
            let save = *old;
            *old = value;
            save
        })
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        self.remove_sync(&key).map(|(_, value)| value)
    }
}

impl index::IndexPin<String> for &'_ scc::TreeIndex<&'static str, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        let guard = scc::Guard::new();
        self.peek(&key, &guard).copied()
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        // NOTE: does not insert if exists
        self.insert_sync(key, value).err().map(|(_, value)| value)
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        self.remove_sync(&key).then_some(0)
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        let guard = scc::Guard::new();
        buffer.extend(
            scc::TreeIndex::range(self, key.., &guard)
                .take(count)
                .map(|(_, value)| *value),
        );
    }
}

impl_index!(String, String);

impl<H: index::Hasher> index::IndexPin<String> for &'_ scc::HashMap<String, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        self.read_sync(key, |_, value| *value)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        self.upsert_sync(key.to_owned(), value)
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        self.update_sync(key, |_, old| {
            let save = *old;
            *old = value;
            save
        })
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        self.remove_sync(key).map(|(_, value)| value)
    }
}

impl index::IndexPin<String> for &'_ scc::TreeIndex<String, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        let guard = scc::Guard::new();
        self.peek(key, &guard).copied()
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        // NOTE: does not insert if exists
        self.insert_sync(key.to_owned(), value)
            .err()
            .map(|(_, value)| value)
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        self.remove_sync(key).then_some(0)
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        let guard = scc::Guard::new();
        buffer.extend(
            scc::TreeIndex::range::<str, _>(
                self,
                (core::ops::Bound::Included(key), core::ops::Bound::Unbounded),
                &guard,
            )
            .take(count)
            .map(|(_, value)| *value),
        );
    }
}
