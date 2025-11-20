use crate::Index;
use crate::index;

impl<H> Index<u64, H> for arctic::concurrent::Map<u64, u64>
where
    H: index::Hasher,
{
    type Send<'a> = &'a arctic::concurrent::Map<u64, u64>;

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

impl<H> index::IndexSend<u64, H> for &'_ arctic::concurrent::Map<u64, u64> {
    type Handle<'a>
        = arctic::concurrent::MapRef<'a, u64, u64>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        arctic::concurrent::Map::pin(self)
    }
}

impl<'a> index::IndexPin<u64> for arctic::concurrent::MapRef<'a, u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        arctic::concurrent::MapRef::get(self, key)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        arctic::concurrent::MapRef::upsert(self, key, value)
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        arctic::concurrent::MapRef::update(self, key, value).ok()
    }

    fn increment(&mut self, key: u64) -> Option<u64> {
        arctic::concurrent::MapRef::upsert_with(self, key, |old| old.unwrap_or(0) + 1)
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
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
