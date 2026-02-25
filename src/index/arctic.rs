use core::ops::ControlFlow;

use crate::Index;
use crate::index;

pub enum Map<K: arctic::concurrent::Key> {
    Disable(arctic::concurrent::Map<K, u64, arctic::concurrent::smr::NoOp>),
    Epoch(arctic::concurrent::Map<K, u64, arctic::concurrent::smr::Epoch>),
    Seize(arctic::concurrent::Map<K, u64, arctic::concurrent::smr::Seize>),
    Hazard(arctic::concurrent::Map<K, u64, arctic::concurrent::smr::Hazard<K::Prefix, u64>>),
}

pub enum MapRef<'a, K: arctic::concurrent::Key> {
    Disable(arctic::concurrent::MapRef<'a, K, u64, arctic::concurrent::smr::NoOp>),
    Epoch(arctic::concurrent::MapRef<'a, K, u64, arctic::concurrent::smr::Epoch>),
    Seize(arctic::concurrent::MapRef<'a, K, u64, arctic::concurrent::smr::Seize>),
    Hazard(arctic::concurrent::MapRef<'a, K, u64, arctic::concurrent::smr::Hazard<K::Prefix, u64>>),
}

impl<K, H> Index<K, H> for Map<K>
where
    K: index::Key + ::arctic::Key,
    H: index::Hasher,
{
    type Send<'a>
        = &'a Map<K>
    where
        K: 'a;

    fn new(config: &index::Config) -> Self {
        match config.smr {
            index::Smr::Disable => Map::Disable(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::NoOp,
            )),
            index::Smr::Epoch => Map::Epoch(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::Epoch::default(),
            )),
            index::Smr::Seize => Map::Seize(arctic::concurrent::Map::with_smr(
                arctic::concurrent::smr::Seize::default(),
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
            Map::Disable(m) => serde_json::to_value(arctic::stat::process(m)).unwrap(),
            Map::Epoch(m) => serde_json::to_value(arctic::stat::process(m)).unwrap(),
            Map::Seize(m) => serde_json::to_value(arctic::stat::process(m)).unwrap(),
            Map::Hazard(m) => serde_json::to_value(arctic::stat::process(m)).unwrap(),
        }
    }

    #[cfg(feature = "stat")]
    fn memory_key_value(&mut self) -> u64 {
        match self {
            Map::Disable(m) => {
                let mut iter = m.as_sequential().iter::<false>();
                let mut total = 0;
                while let Some((key, _)) = iter.lend() {
                    total += <K as ::arctic::raw::Key>::len(key) + 8;
                }
                total as u64
            }
            Map::Epoch(m) => {
                let mut iter = m.as_sequential().iter::<false>();
                let mut total = 0;
                while let Some((key, _)) = iter.lend() {
                    total += <K as ::arctic::raw::Key>::len(key) + 8;
                }
                total as u64
            }
            Map::Seize(m) => {
                let mut iter = m.as_sequential().iter::<false>();
                let mut total = 0;
                while let Some((key, _)) = iter.lend() {
                    total += <K as ::arctic::raw::Key>::len(key) + 8;
                }
                total as u64
            }
            Map::Hazard(m) => {
                let mut iter = m.as_sequential().iter::<false>();
                let mut total = 0;
                while let Some((key, _)) = iter.lend() {
                    total += <K as ::arctic::raw::Key>::len(key) + 8;
                }
                total as u64
            }
        }
    }
}

impl<K, H> index::IndexSend<K, H> for &'_ Map<K>
where
    K: index::Key + ::arctic::Key,
{
    type Handle<'a>
        = MapRef<'a, K>
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

impl<'a, K> index::IndexPin<K> for MapRef<'a, K>
where
    K: index::Key + ::arctic::Key,
{
    fn enable_membarrier(&self) {
        if let MapRef::Hazard(m) = self {
            m.smr().enable_membarrier();
        }
    }

    fn get(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        match self {
            MapRef::Disable(r) => arctic::concurrent::MapRef::get(r, key).as_deref().copied(),
            MapRef::Epoch(r) => arctic::concurrent::MapRef::get(r, key).as_deref().copied(),
            MapRef::Seize(r) => arctic::concurrent::MapRef::get(r, key).as_deref().copied(),
            MapRef::Hazard(r) => arctic::concurrent::MapRef::get(r, key).as_deref().copied(),
        }
    }

    fn insert(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        match self {
            MapRef::Disable(r) => arctic::concurrent::MapRef::upsert(r, key, value)
                .as_deref()
                .copied(),
            MapRef::Epoch(r) => arctic::concurrent::MapRef::upsert(r, key, value)
                .as_deref()
                .copied(),
            MapRef::Seize(r) => arctic::concurrent::MapRef::upsert(r, key, value)
                .as_deref()
                .copied(),
            MapRef::Hazard(r) => arctic::concurrent::MapRef::upsert(r, key, value)
                .as_deref()
                .copied(),
        }
    }

    fn update(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        match self {
            MapRef::Disable(r) => arctic::concurrent::MapRef::update(r, key, value)
                .ok()
                .as_deref()
                .copied(),
            MapRef::Epoch(r) => arctic::concurrent::MapRef::update(r, key, value)
                .ok()
                .as_deref()
                .copied(),
            MapRef::Seize(r) => arctic::concurrent::MapRef::update(r, key, value)
                .ok()
                .as_deref()
                .copied(),
            MapRef::Hazard(r) => arctic::concurrent::MapRef::update(r, key, value)
                .ok()
                .as_deref()
                .copied(),
        }
    }

    fn remove(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        match self {
            MapRef::Disable(r) => arctic::concurrent::MapRef::remove(r, key)
                .as_deref()
                .copied(),
            MapRef::Epoch(r) => arctic::concurrent::MapRef::remove(r, key)
                .as_deref()
                .copied(),
            MapRef::Seize(r) => arctic::concurrent::MapRef::remove(r, key)
                .as_deref()
                .copied(),
            MapRef::Hazard(r) => arctic::concurrent::MapRef::remove(r, key)
                .as_deref()
                .copied(),
        }
    }

    fn scan(
        &mut self,
        key: <K as arctic::raw::Key>::Borrow<'static>,
        mut count: usize,
        buffer: &mut Vec<u64>,
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
                            buffer.push(value);
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
                            buffer.push(value);
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
                            buffer.push(value);
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
                            buffer.push(value);
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
