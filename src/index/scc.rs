use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for scc::HashMap<K, u32, H> {
    type Handle<'a> = &'a scc::HashMap<K, u32, H>;

    fn new() -> Self {
        scc::HashMap::with_hasher(H::default())
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::Handle<K> for &'_ scc::HashMap<K, u32, H> {
    fn get(&mut self, key: &K) -> Option<u32> {
        self.read_sync(key, |_, value| *value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        self.insert_sync(key, value).err().map(|(_, value)| value)
    }
}
