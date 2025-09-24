use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for dashmap::DashMap<K, u32, H> {
    type Handle<'a> = &'a Self;

    fn new() -> Self {
        dashmap::DashMap::with_hasher(H::default())
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::Handle<K> for &'_ dashmap::DashMap<K, u32, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        dashmap::DashMap::get(self, key).map(|value| *value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        dashmap::DashMap::insert(self, key, value)
    }
}
