use crate::Index;
use crate::index;

impl<K, H> Index<K, H> for arctic::Map<K, u32>
where
    K: index::Key,
    H: index::Hasher,
{
    type Handle<'a> = arctic::MapRef<'a, K, u32>;

    fn new() -> Self {
        arctic::Map::default()
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self.pin()
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::process(self)).unwrap()
    }
}

impl<'a, K> index::Handle<K> for arctic::MapRef<'a, K, u32>
where
    K: index::Key,
{
    fn get(&mut self, key: &K) -> Option<u32> {
        arctic::MapRef::get(self, key)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        arctic::MapRef::insert(self, &key, value)
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
