use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for concurrent_map::ConcurrentMap<u64, u64> {
    type Send<'a> = Self;

    fn new(_: &index::Config) -> Self {
        concurrent_map::ConcurrentMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self.clone()
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for concurrent_map::ConcurrentMap<u64, u64> {
    type Handle<'a> = &'a Self;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64> for &'_ concurrent_map::ConcurrentMap<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::get(self, &key)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }
}
