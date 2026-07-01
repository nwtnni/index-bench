use core::ops::ControlFlow;

use crate::Index;
use crate::index;

#[cfg(feature = "smr-hazard")]
type Smr = arctic::concurrent::smr::Hazard;

#[cfg(feature = "smr-disable")]
type Smr = arctic::concurrent::smr::NoOp;

#[cfg(feature = "smr-epoch")]
type Smr = arctic::concurrent::smr::Epoch;

// FIXME
#[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-hazard")))]
type Smr = arctic::concurrent::smr::NoOp;

pub type Map<K, V> = arctic::concurrent::Map<K, V, Smr>;

macro_rules! impl_index {
    ($bench:ty, $arctic:ty $(, $convert:expr)?) => {
        impl<V, H> Index<$bench, V, H> for Map<$arctic, V>
        where
            V: ::arctic::concurrent::Value + Send + Sync,
            H: index::Hasher,
        {
            type Send<'a>
                = &'a Map<$arctic, V>
            where
                V: 'a;

            fn new(_config: &index::Config) -> Self {
                #[cfg(feature = "smr-hazard")]
                {
                    Map::with_smr(Box::new(
                        arctic::concurrent::smr::hazard::Global::default()
                            .with_reclaim_threshold(_config.reclaim_threshold),
                    ))
                }

                #[cfg(feature = "smr-disable")]
                {
                    Map::with_smr(arctic::concurrent::smr::NoOp)
                }

                #[cfg(feature = "smr-epoch")]
                {
                    Map::with_smr(Box::new(
                        arctic::concurrent::smr::epoch::Global::with_bag_capacity(_config.reclaim_threshold),
                    ))
                }

                // FIXME
                #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-hazard")))]
                {
                    Map::with_smr(arctic::concurrent::smr::NoOp)
                    // Map::with_smr(arctic::concurrent::smr::Seize::default())
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
                    total += crate::index::Key::with_slice(&key, |slice| slice.len()) + 8;
                }
                total as u64
            }

            #[cfg(feature = "stat-garbage")]
            fn garbage(&mut self) -> u32 {
                <
                    <Smr as arctic::concurrent::Smr>::Global<<K as arctic::concurrent::Key>::Prefix, V>
                    as arctic::concurrent::smr::Global<<K as arctic::concurrent::Key>::Prefix, V>
                >::garbage(self.smr_mut())
            }
        }

        impl<V, H> index::IndexSend<$bench, V, H> for &'_ Map<$arctic, V>
        where
            V: ::arctic::concurrent::Value + Send + Sync,
        {
            type Handle<'a>
                = &'a Map<$arctic, V>
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }

        impl<V> index::IndexPin<$bench, V> for &'_ Map<$arctic, V>
        where
            V: ::arctic::concurrent::Value + Send + Sync,
        {
            fn enable_membarrier(&self) {
                // #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-seize")))]
                // self.smr().enable_membarrier();
            }

            fn get(&mut self, key: $bench) {
                $(let key = ($convert)(key);)?
                let _ = core::hint::black_box(Map::get(self, &key));
            }

            fn insert(&mut self, key: $bench, value: V) {
                $(let key = ($convert)(key);)?
                let _ = core::hint::black_box(Map::upsert(self, key, value));
            }

            fn update(&mut self, key: $bench, value: V) {
                $(let key = ($convert)(key);)?
                let _ = core::hint::black_box(Map::update(self, &key, value));
            }

            fn remove(&mut self, key: $bench) {
                $(let key = ($convert)(key);)?
                let _ = core::hint::black_box(Map::remove_non_recursive(self, &key));
            }

            fn scan(&mut self, key: $bench, mut count: usize) {
                $(let key = ($convert)(key);)?
                let shard = Map::range(self, key..);

                shard
                    .values::<arctic::Ascend>()
                    .for_each_internal(|_| {
                        if count == 0 {
                            ControlFlow::Break(())
                        } else {
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
    }
}

impl_index!(u64, u64);
impl_index!(u128, u128);
impl_index!(
    &'static [u8],
    ::arctic::key::BoxedSlice<::arctic::key::Terminated<b'\n'>>,
    |key: &'static [u8]| unsafe {
        ::arctic::key::Slice::<::arctic::key::Terminated<b'\n'>>::new_unchecked(key)
    }
);
impl_index!(
    &'static [u8],
    &'static ::arctic::key::Slice<::arctic::key::Terminated<b'\n'>>,
    |key: &'static [u8]| unsafe {
        ::arctic::key::Slice::<::arctic::key::Terminated<b'\n'>>::new_unchecked(key)
    }
);

impl index::Key for &'_ ::arctic::key::Slice<::arctic::key::Terminated<b'\n'>> {
    fn with_slice<F, T>(&self, with: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        with(self.as_bytes())
    }
}
