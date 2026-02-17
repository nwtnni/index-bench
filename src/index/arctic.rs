use core::ops::ControlFlow;

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

    fn new(_config: &index::Config) -> Self {
        arctic::concurrent::Map::with_smr(arctic::concurrent::smr::Seize::default())
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::process(self)).unwrap()
    }

    #[cfg(feature = "stat")]
    fn memory_key_value(&mut self) -> u64 {
        let mut iter = self.as_sequential().iter::<false>();
        let mut total = 0;
        while let Some((key, _)) = iter.lend() {
            total += <K as ::arctic::raw::Key>::len(key) + 8;
        }
        total as u64
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
    // fn enable_membarrier(&self) {
    //     self.smr().enable_membarrier();
    // }

    fn get(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        arctic::concurrent::MapRef::get(self, key)
            .as_deref()
            .copied()
    }

    fn insert(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        arctic::concurrent::MapRef::upsert(self, key, value)
            .as_deref()
            .copied()
    }

    fn update(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        arctic::concurrent::MapRef::update(self, key, value)
            .ok()
            .as_deref()
            .copied()
    }

    fn remove(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        arctic::concurrent::MapRef::remove(self, key)
            .as_deref()
            .copied()
    }

    fn scan(
        &mut self,
        key: <K as arctic::raw::Key>::Borrow<'static>,
        mut count: usize,
        buffer: &mut Vec<u64>,
    ) {
        let Some(prefix) = arctic::concurrent::MapRef::range(self, key..) else {
            return;
        };

        prefix
            .values::<arctic::Ascend>()
            .for_each_internal(|value| {
                if count == 0 {
                    ControlFlow::Break(())
                } else {
                    buffer.push(value);
                    count -= 1;
                    ControlFlow::Continue(())
                }
            })
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
