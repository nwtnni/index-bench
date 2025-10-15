use crate::Index;
use crate::index;

impl<K, H> Index<K, H> for arctic::concurrent::Map<K, u32>
where
    K: index::Key,
    H: index::Hasher,
{
    type Send<'a> = &'a arctic::concurrent::Map<K, u32>;

    fn new() -> Self {
        arctic::concurrent::Map::default()
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
        arctic::concurrent::MapRef::insert(self, key.borrow(), value)
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        arctic::concurrent::MapRef::update(self, key.borrow(), value)
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        arctic::concurrent::MapRef::remove(self, key.borrow())
    }

    fn range<'b>(&'b mut self, min: &'b K, max: &'b K, output: &mut Vec<(K, u32)>) {
        if cfg!(feature = "range-optimistic") {
            arctic::concurrent::MapRef::range_optimistic(self, min.borrow(), max.borrow(), output)
        } else if cfg!(feature = "range-pessimistic") {
            arctic::concurrent::MapRef::range_pessimistic(self, min.borrow(), max.borrow())
                .for_each(|key, value| output.push((K::from(key), value)));
        } else {
            arctic::concurrent::MapRef::range_non_linearizable(self, min.borrow(), max.borrow())
                .for_each(|key, value| output.push((K::from(key), value)));
        }
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
