use crate::Index;
use crate::index;

pub struct Map<K: index::Key>(concurrent_map::ConcurrentMap<K, u32>);

impl<K: index::Key> Index<K> for Map<K> {
    type Handle = Self;

    fn new() -> Self {
        Self(concurrent_map::ConcurrentMap::new())
    }

    fn pin(&self) -> Self::Handle {
        Self(self.0.clone())
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        self.0.get(key)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        self.0.insert(key, value)
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
