use crate::Index;
use crate::index;

pub struct Map<K: index::Key>(concurrent_map::ConcurrentMap<K, u32>);

/// HACK: the benchmark runner ensures that `Index::pin` will
/// be called in each thread before beginning operation, so
/// this should be fine.
unsafe impl<K: index::Key> Sync for Map<K> {}

impl<K: index::Key, H: index::Hasher> Index<K, H> for Map<K> {
    type Handle<'a> = concurrent_map::ConcurrentMap<K, u32>;

    fn new() -> Self {
        Self(concurrent_map::ConcurrentMap::new())
    }

    fn pin(&self) -> Self::Handle<'static> {
        self.0.clone()
    }
}

impl<K: index::Key> index::Handle<K> for concurrent_map::ConcurrentMap<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        concurrent_map::ConcurrentMap::get(self, key)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        concurrent_map::ConcurrentMap::insert(self, key, value)
    }
}
