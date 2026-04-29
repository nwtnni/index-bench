use core::borrow::Borrow as _;
use core::ops::ControlFlow;

use crate::Index;
use crate::index;
use arctic::concurrent::smr::Global as _;

#[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
type Smr = arctic::concurrent::smr::Hazard;

#[cfg(feature = "smr-disable")]
type Smr = arctic::concurrent::smr::NoOp;

#[cfg(feature = "smr-epoch")]
type Smr = arctic::concurrent::smr::Epoch;

#[cfg(feature = "smr-seize")]
type Smr = arctic::concurrent::smr::Seize;

pub type Map<K, V> = arctic::concurrent::Map<K, V, Smr>;

impl<K, V, H> Index<K, V, H> for Map<K, V>
where
    K: index::Key + ::arctic::Key,
    V: index::Value + ::arctic::Value + Send + Sync,
    H: index::Hasher,
{
    type Send<'a>
        = &'a Map<K, V>
    where
        K: 'a,
        V: 'a;

    fn new(config: &index::Config) -> Self {
        #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
        {
            Map::with_smr(Box::new(
                arctic::concurrent::smr::hazard::Global::default()
                    .with_reclaim_threshold(config.reclaim_threshold),
            ))
        }

        #[cfg(feature = "smr-disable")]
        {
            Map::with_smr(arctic::concurrent::smr::NoOp)
        }

        #[cfg(feature = "smr-epoch")]
        {
            Map::with_smr(Box::new(
                arctic::concurrent::smr::epoch::Global::with_bag_capacity(config.reclaim_threshold),
            ))
        }

        #[cfg(feature = "smr-seize")]
        {
            Map::with_smr(arctic::concurrent::smr::seize::Global::with_batch_size(
                config.reclaim_threshold,
            ))
        }
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
        let mut iter = self.as_sequential().all().entries::<arctic::Ascend>();
        let mut total = 0;
        while let Some((key, _)) = iter.lend() {
            total += <K as ::arctic::raw::Key>::len(key) + 8;
        }
        total as u64
    }

    #[cfg(feature = "stat-garbage")]
    fn garbage(&mut self) -> u32 {
        self.smr().garbage()
    }
}

impl<K, V, H> index::IndexSend<K, V, H> for &'_ Map<K, V>
where
    K: index::Key + ::arctic::Key,
    V: index::Value + ::arctic::Value + Send + Sync,
{
    type Handle<'a>
        = &'a Map<K, V>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<'a, K, V> index::IndexPin<K, V> for &'a Map<K, V>
where
    K: index::Key + ::arctic::Key,
    V: index::Value + ::arctic::Value + Send + Sync,
{
    fn enable_membarrier(&self) {
        #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
        self.smr().enable_membarrier();
    }

    fn get(&mut self, key: <K as index::Key>::Borrow) -> Option<V> {
        let _ = std::hint::black_box(Map::get(self, key.borrow()));
        None
    }

    fn insert(&mut self, key: <K as index::Key>::Borrow, value: V) -> Option<V> {
        let _ = std::hint::black_box(Map::upsert(self, key.borrow(), value));
        None
    }

    fn update(&mut self, key: <K as index::Key>::Borrow, value: V) -> Option<V> {
        let _ = std::hint::black_box(Map::update(self, key.borrow(), value));
        None
    }

    fn remove(&mut self, key: <K as index::Key>::Borrow) -> Option<V> {
        let _ = std::hint::black_box(Map::remove_non_recursive(self, key.borrow()));
        None
    }

    fn scan(&mut self, key: <K as index::Key>::Borrow, mut count: usize, buffer: &mut Vec<V>) {
        let Some(prefix) = Map::range(self, key.borrow()..) else {
            return;
        };

        prefix
            .values::<arctic::Ascend>()
            .for_each_internal(|value| {
                if count == 0 {
                    ControlFlow::Break(())
                } else {
                    buffer.push(V::from_borrow(value));
                    count -= 1;
                    ControlFlow::Continue(())
                }
            });
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
