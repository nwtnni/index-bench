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
    fn get(&mut self, key: u64) -> Option<u64> {
        let map = self.pin();
        map.get(&key).copied()
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        let map = self.pin();
        map.insert(key, value).copied()
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        let map = self.pin();
        map.update(key, |_| value).copied()
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        let map = self.pin();
        map.remove(&key).copied()
    }
}

impl_index!(String, &'static str);

impl<H: index::Hasher> index::IndexPin<String, u64> for &'_ papaya::HashMap<&'static str, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        let map = self.pin();
        map.get(&key).copied()
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        let map = self.pin();
        map.insert(key, value).copied()
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        let map = self.pin();
        map.update(key, |_| value).copied()
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        let map = self.pin();
        map.remove(&key).copied()
    }
}

impl_index!(String, String);

impl<H: index::Hasher> index::IndexPin<String, u64> for &'_ papaya::HashMap<String, u64, H> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        let map = self.pin();
        map.get(key).copied()
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        let map = self.pin();
        map.insert(key.to_owned(), value).copied()
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        let map = self.pin();
        map.update(key.to_owned(), |_| value).copied()
    }

    fn remove(&mut self, key: &'static str) -> Option<u64> {
        let map = self.pin();
        map.remove(key).copied()
    }
}
