use core::ops::ControlFlow;

use crate::Index;
use crate::index;

pub enum Map<K: index::Key, V: index::Value> {
    Disable(arctic::concurrent::Map<K, V, arctic::concurrent::smr::NoOp>),
    Epoch(arctic::concurrent::Map<K, V, arctic::concurrent::smr::Epoch>),
    Seize(arctic::concurrent::Map<K, V, arctic::concurrent::smr::Seize>),
    Hazard(arctic::concurrent::Map<K, V, arctic::concurrent::smr::Hazard<K::Prefix, V>>),
}

pub enum MapRef<'a, K: index::Key, V: index::Value> {
    Disable(arctic::concurrent::MapRef<'a, K, V, arctic::concurrent::smr::NoOp>),
    Epoch(arctic::concurrent::MapRef<'a, K, V, arctic::concurrent::smr::Epoch>),
    Seize(arctic::concurrent::MapRef<'a, K, V, arctic::concurrent::smr::Seize>),
    Hazard(arctic::concurrent::MapRef<'a, K, V, arctic::concurrent::smr::Hazard<K::Prefix, V>>),
}

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
        match config.smr {
            index::Smr::Disable => Map::Disable(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::NoOp,
            )),
            index::Smr::Epoch => Map::Epoch(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::Epoch::with_bag_capacity(config.reclaim_threshold),
            )),
            index::Smr::Seize => Map::Seize(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::Seize::with_batch_size(config.reclaim_threshold),
            )),
            index::Smr::Hazard => Map::Hazard(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::Hazard::default()
                    .with_reclaim_threshold(config.reclaim_threshold),
            )),
        }
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        match self {
            Map::Disable(_) | Map::Epoch(_) | Map::Seize(_) => serde_json::Value::Null,
            Map::Hazard(m) => serde_json::to_value(arctic::stat::process(m)).unwrap(),
        }
    }

    #[cfg(feature = "stat")]
    fn memory_key_value(&mut self) -> u64 {
        match self {
            Map::Disable(_) | Map::Epoch(_) | Map::Seize(_) => 0,
            Map::Hazard(m) => {
                // `iter::<false>` corresponds to `arctic::Descend` in legacy code, I think.
                // https://github.com/nwtnni/arctic/blob/4416d06259a086088c31e1ee332fc3e11e846859/src/sequential.rs#L132
                let mut iter = m.as_sequential().all().entries::<arctic::Descend>();
                let mut total = 0;
                while let Some((key, _)) = iter.lend() {
                    total += <K as ::arctic::raw::Key>::len(key) + 8;
                }
                total as u64
            }
        }
    }
}

impl<K, V, H> index::IndexSend<K, V, H> for &'_ Map<K, V>
where
    K: index::Key + ::arctic::Key,
    V: index::Value + ::arctic::Value + Send + Sync,
{
    type Handle<'a>
        = MapRef<'a, K, V>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        match self {
            Map::Disable(m) => MapRef::Disable(m.pin()),
            Map::Epoch(m) => MapRef::Epoch(m.pin()),
            Map::Seize(m) => MapRef::Seize(m.pin()),
            Map::Hazard(m) => MapRef::Hazard(m.pin()),
        }
    }
}

impl<'a, K, V> index::IndexPin<K, V> for MapRef<'a, K, V>
where
    K: index::Key + ::arctic::Key,
    V: index::Value + ::arctic::Value + Send + Sync,
{
    fn enable_membarrier(&self) {
        if let MapRef::Hazard(m) = self {
            m.smr().enable_membarrier();
        }
    }

    fn get(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<V> {
        match self {
            MapRef::Disable(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::get(r, key));
                None
            }
            MapRef::Epoch(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::get(r, key));
                None
            }
            MapRef::Seize(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::get(r, key));
                None
            }
            MapRef::Hazard(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::get(r, key));
                None
            }
        }
    }

    fn insert(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>, value: V) -> Option<V> {
        match self {
            MapRef::Disable(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::upsert(r, key, value));
                None
            }
            MapRef::Epoch(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::upsert(r, key, value));
                None
            }
            MapRef::Seize(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::upsert(r, key, value));
                None
            }
            MapRef::Hazard(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::upsert(r, key, value));
                None
            }
        }
    }

    fn update(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>, value: V) -> Option<V> {
        match self {
            MapRef::Disable(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::update(r, key, value));
                None
            }
            MapRef::Epoch(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::update(r, key, value));
                None
            }
            MapRef::Seize(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::update(r, key, value));
                None
            }
            MapRef::Hazard(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::update(r, key, value));
                None
            }
        }
    }

    fn remove(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<V> {
        match self {
            MapRef::Disable(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::remove(r, key));
                None
            }
            MapRef::Epoch(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::remove(r, key));
                None
            }
            MapRef::Seize(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::remove(r, key));
                None
            }
            MapRef::Hazard(r) => {
                std::hint::black_box(arctic::concurrent::MapRef::remove(r, key));
                None
            }
        }
    }

    fn scan(
        &mut self,
        key: <K as arctic::raw::Key>::Borrow<'static>,
        mut count: usize,
        buffer: &mut Vec<V>,
    ) {
        match self {
            MapRef::Disable(r) => {
                let Some(prefix) = arctic::concurrent::MapRef::range(r, key..) else {
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
            MapRef::Epoch(r) => {
                let Some(prefix) = arctic::concurrent::MapRef::range(r, key..) else {
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
            MapRef::Seize(r) => {
                let Some(prefix) = arctic::concurrent::MapRef::range(r, key..) else {
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
            MapRef::Hazard(r) => {
                let Some(prefix) = arctic::concurrent::MapRef::range(r, key..) else {
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
        }
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }
}
