use crate::Index;
use crate::index;

impl<K, H> Index<K, H> for arctic::concurrent::Map<K, u64>
where
    K: index::Key + ::arctic::Key,
    H: index::Hasher,
{
    type Send<'a>
        = &'a arctic::concurrent::Map<K, u64>
    where
        K: 'a;

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

impl<K, H> index::IndexSend<K, H> for &'_ arctic::concurrent::Map<K, u64>
where
    K: index::Key + ::arctic::Key,
{
    type Handle<'a>
        = arctic::concurrent::MapRef<'a, K, u64>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        arctic::concurrent::Map::pin(self)
    }
}

impl<'a, K> index::IndexPin<K> for arctic::concurrent::MapRef<'a, K, u64>
where
    K: index::Key + ::arctic::Key,
{
    fn get(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        arctic::concurrent::MapRef::get(self, key)
    }

    fn insert(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        arctic::concurrent::MapRef::upsert(self, key, value)
    }

    fn update(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        arctic::concurrent::MapRef::update(self, key, value).ok()
    }

    fn remove(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        arctic::concurrent::MapRef::remove(self, key)
    }

    // fn range<'b>(
    //     &'b mut self,
    //     _retry_scan: usize,
    //     min: &'b u64,
    //     max: &'b u64,
    //     output: &mut Vec<(K, u64)>,
    // ) {
    //     let Some(guard) = arctic::concurrent::MapRef::range(self, min.borrow(), max.borrow())
    //     else {
    //         return;
    //     };
    //
    //     guard
    //         .entries::<arctic::iter::Sorted>()
    //         .for_each(|key, value| output.push((K::clone_from_borrow(key), value)));
    // }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
