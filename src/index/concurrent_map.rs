use core::ops::RangeBounds;

use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for concurrent_map::ConcurrentMap<K, u32> {
    type Send<'a> = Self;

    fn new() -> Self {
        concurrent_map::ConcurrentMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self.clone()
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H>
    for concurrent_map::ConcurrentMap<K, u32>
{
    type Handle<'a> = &'a Self;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<'a, K: index::Key> index::IndexPin<K> for &'a concurrent_map::ConcurrentMap<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        concurrent_map::ConcurrentMap::get(self, key)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }

    fn range<'b, R: RangeBounds<&'b K>>(
        &'b mut self,
        range: R,
    ) -> impl Iterator<Item = (K, u32)> + 'b {
        let start = range.start_bound().map(|start| (**start).clone());
        let end = range.end_bound().map(|end| (**end).clone());
        concurrent_map::ConcurrentMap::range(self, (start, end))
    }
}
