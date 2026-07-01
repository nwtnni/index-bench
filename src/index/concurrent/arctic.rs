use core::marker::PhantomData;
use core::num::NonZeroU64;
use core::ops::ControlFlow;
use core::sync::atomic::Ordering;

use seize::Guard as _;

use crate::Index;
use crate::index;

#[cfg(feature = "smr-hazard")]
type Smr<K, V> = arctic::concurrent::smr::Hazard<K, V>;

#[cfg(feature = "smr-disable")]
type Smr<K, V> = NoOp<K, V>;

#[cfg(feature = "smr-epoch")]
type Smr<K, V> = Epoch<K, V>;

#[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-hazard")))]
type Smr<K, V> = Seize<K, V>;

pub type Map<K, V> = arctic::concurrent::Map<K, V, Smr<K, V>>;

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
                    Map::with_smr(
                        arctic::concurrent::smr::Hazard::default()
                            .with_reclaim_threshold(_config.reclaim_threshold),
                    )
                }

                #[cfg(feature = "smr-disable")]
                {
                    Map::with_smr(NoOp(PhantomData))
                }

                #[cfg(feature = "smr-epoch")]
                {
                    crossbeam_epoch::set_bag_capacity(_config.reclaim_threshold);
                    Map::with_smr(Epoch::default())
                }

                #[cfg(not(any(feature = "smr-disable", feature = "smr-epoch", feature = "smr-hazard")))]
                {
                    Map::with_smr(Seize { collector: seize::Collector::default().batch_size(_config.reclaim_threshold), _type: PhantomData})
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
                ::arctic::concurrent::Smr::garbage(self.smr_mut())
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

pub struct NoOp<K, V>(PhantomData<(K, V)>);

impl<K: ::arctic::Key, V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Smr<K, V>
    for NoOp<K, V>
{
    type Guard<'g>
        = NoOp<K, V>
    where
        V: 'g,
        Self: 'g;

    fn guard<'g>(&'g self, _: K::Read<'_>) -> Self::Guard<'g>
    where
        V: 'g,
    {
        Self(PhantomData)
    }

    fn garbage(&self) -> u32 {
        ::arctic::concurrent::smr::Smr::<K, V>::garbage(&::arctic::concurrent::smr::NoOp)
    }
}

impl<K: ::arctic::Key, V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Guard<V>
    for NoOp<K, V>
{
    unsafe fn retire_node(&mut self, bits: usize, node: std::num::NonZeroU64) {
        unsafe {
            ::arctic::concurrent::smr::Guard::retire_node(
                &mut ::arctic::concurrent::smr::no_op::Guard::<(), V>::default(),
                bits,
                node,
            )
        }
    }

    unsafe fn retire_value(&mut self, value: u64) {
        unsafe {
            ::arctic::concurrent::smr::Guard::retire_value(
                &mut ::arctic::concurrent::smr::no_op::Guard::<(), V>::default(),
                value,
            )
        }
    }
}

pub struct Epoch<K, V>(PhantomData<(K, V)>);

impl<K, V> Default for Epoch<K, V> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<K: ::arctic::Key, V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Smr<K, V>
    for Epoch<K, V>
{
    type Guard<'g>
        = EpochGuard<V>
    where
        V: 'g,
        Self: 'g;

    fn guard<'g>(&'g self, _: K::Read<'_>) -> Self::Guard<'g>
    where
        V: 'g,
    {
        EpochGuard {
            guard: crossbeam_epoch::pin(),
            _type: PhantomData,
        }
    }

    fn garbage(&self) -> u32 {
        crossbeam_epoch::GLOBAL_GARBAGE_COUNT.load(Ordering::Relaxed) as u32
    }
}

pub struct EpochGuard<V> {
    guard: crossbeam_epoch::Guard,
    _type: PhantomData<V>,
}

impl<V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Guard<V> for EpochGuard<V> {
    unsafe fn retire_node(&mut self, _: usize, node: std::num::NonZeroU64) {
        self.guard
            .defer(move || unsafe { ::arctic::concurrent::smr::deallocate_node(node) })
    }

    unsafe fn retire_value(&mut self, value: u64) {
        self.guard
            .defer(move || unsafe { ::arctic::concurrent::smr::deallocate_value::<V>(value) })
    }
}

pub struct Seize<K, V> {
    collector: seize::Collector,
    _type: PhantomData<(K, V)>,
}

impl<K, V> Default for Seize<K, V> {
    fn default() -> Self {
        Self {
            collector: Default::default(),
            _type: PhantomData,
        }
    }
}

impl<K: ::arctic::Key, V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Smr<K, V>
    for Seize<K, V>
{
    type Guard<'g>
        = SeizeGuard<'g, V>
    where
        V: 'g,
        Self: 'g;

    fn guard<'g>(&'g self, _: K::Read<'_>) -> Self::Guard<'g>
    where
        V: 'g,
    {
        SeizeGuard {
            guard: self.collector.enter(),
            _type: PhantomData,
        }
    }

    fn garbage(&self) -> u32 {
        self.collector.garbage()
    }
}

pub struct SeizeGuard<'g, V> {
    guard: seize::LocalGuard<'g>,
    _type: PhantomData<V>,
}

impl<V: ::arctic::concurrent::Value> ::arctic::concurrent::smr::Guard<V> for SeizeGuard<'_, V> {
    unsafe fn retire_node(&mut self, _: usize, node: std::num::NonZeroU64) {
        unsafe {
            self.guard
                .defer_retire(node.get() as *mut (), move |node, _| {
                    ::arctic::concurrent::smr::deallocate_node(NonZeroU64::new_unchecked(
                        node as u64,
                    ))
                })
        }
    }

    unsafe fn retire_value(&mut self, value: u64) {
        unsafe {
            self.guard.defer_retire(value as *mut (), move |value, _| {
                ::arctic::concurrent::smr::deallocate_value::<V>(value as u64)
            })
        }
    }
}
