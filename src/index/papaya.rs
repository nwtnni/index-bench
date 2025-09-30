use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for papaya::HashMap<K, u32, H> {
    type Handle<'a> = &'a Self;

    fn new() -> Self {
        papaya::HashMap::with_hasher(H::default())
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::Handle<K> for &'_ papaya::HashMap<K, u32, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let map = self.pin();
        map.get(key).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let map = self.pin();
        map.insert(key, value).copied()
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        let map = self.pin();
        map.remove(&key).copied()
    }
}
