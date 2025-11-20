use crate::Index;
use crate::index;

impl<K, H> Index<K, H> for arctic::concurrent::Map<K, u32>
where
    K: index::Key,
    H: index::Hasher,
{
    type Send<'a> = &'a arctic::concurrent::Map<K, u32>;

    fn new(config: &index::Config) -> Self {
        arctic::concurrent::Map::with_reclaim_threshold(config.reclaim_threshold)
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::process(self)).unwrap()
    }
}

impl<K, H> index::IndexSend<K, H> for &'_ arctic::concurrent::Map<K, u32>
where
    K: index::Key,
    H: index::Hasher,
{
    type Handle<'a>
        = arctic::concurrent::MapRef<'a, K, u32>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        arctic::concurrent::Map::pin(self)
    }
}

impl<'a, K> index::IndexPin<K> for arctic::concurrent::MapRef<'a, K, u32>
where
    K: index::Key,
{
    fn get(&mut self, key: &K) -> Option<u32> {
        arctic::concurrent::MapRef::get(self, key.borrow())
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        arctic::concurrent::MapRef::upsert(self, key.borrow(), value)
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        arctic::concurrent::MapRef::update(self, key.borrow(), value).ok()
    }

    fn increment(&mut self, key: K) -> Option<u32> {
        arctic::concurrent::MapRef::upsert_with(self, key.borrow(), |old| old.unwrap_or(0) + 1)
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        arctic::concurrent::MapRef::remove(self, key.borrow())
    }

    fn range<'b>(
        &'b mut self,
        _retry_scan: usize,
        min: &'b K,
        max: &'b K,
        output: &mut Vec<(K, u32)>,
    ) {
        let Some(guard) = arctic::concurrent::MapRef::range(self, min.borrow(), max.borrow())
        else {
            return;
        };

        guard
            .entries::<arctic::iter::Sorted>()
            .for_each(|key, value| output.push((K::clone_from_borrow(key), value)));
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
