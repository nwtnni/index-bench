use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for concurrent_map::ConcurrentMap<K, u64> {
    type Send<'a> = Self;

    fn new(_: &index::Config) -> Self {
        concurrent_map::ConcurrentMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self.clone()
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H>
    for concurrent_map::ConcurrentMap<K, u64>
{
    type Handle<'a> = &'a Self;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ concurrent_map::ConcurrentMap<K, u64> {
    fn get(&mut self, key: &K) -> Option<u64> {
        concurrent_map::ConcurrentMap::get(self, key)
    }

    fn insert(&mut self, key: K, value: u64) -> Option<u64> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }

    fn range<'b>(
        &'b mut self,
        _retry_scan: usize,
        min: &'b K,
        max: &'b K,
        output: &mut Vec<(K, u64)>,
    ) {
        output.extend(concurrent_map::ConcurrentMap::range(self, min..=max));
    }
}
