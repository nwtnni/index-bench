use core::ops::ControlFlow;

use crate::Index;
use crate::index;

#[cfg(feature = "smr-disable")]
pub type DefaultSmr<K> = arctic::concurrent::smr::NoOp;

#[cfg(feature = "smr-epoch")]
pub type DefaultSmr<K> = arctic::concurrent::smr::Epoch;

#[cfg(feature = "smr-seize")]
pub type DefaultSmr<K> = arctic::concurrent::smr::Seize;

#[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
pub type DefaultSmr<K> = arctic::concurrent::smr::Hazard<<K as arctic::Key>::Prefix, u64>;

fn resolve_smr<K>(config: &index::Config) -> DefaultSmr<K>
where
    K: index::Key + arctic::Key,
{
    let mut smr = DefaultSmr::<K>::default();
    #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
    {
        smr = smr.with_reclaim_threshold(config.reclaim_threshold);
    }
    smr
}

impl<K, H> Index<K, H> for arctic::concurrent::Map<K, u64, DefaultSmr<K>>
where
    K: index::Key + ::arctic::Key,
    H: index::Hasher,
{
    type Send<'a>
        = &'a arctic::concurrent::Map<K, u64, DefaultSmr<K>>
    where
        K: 'a;

    fn new(config: &index::Config) -> Self {
        arctic::concurrent::Map::with_smr(resolve_smr::<K>(config))
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

impl<K, H> index::IndexSend<K, H> for &'_ arctic::concurrent::Map<K, u64, DefaultSmr<K>>
where
    K: index::Key + ::arctic::Key,
{
    type Handle<'a>
        = arctic::concurrent::MapRef<'a, K, u64, DefaultSmr<K>>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        arctic::concurrent::Map::pin(self)
    }
}

impl<'a, K> index::IndexPin<K> for arctic::concurrent::MapRef<'a, K, u64, DefaultSmr<K>>
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
