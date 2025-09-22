use core::hash::Hash;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

pub mod concurrent_map;
pub mod crossbeam_skiplist;
pub mod papaya;
pub mod scc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Config {
    Arctic,
    ConcurrentMap,
    CrossbeamSkiplist,
    Papaya,
    Scc,
}

pub trait Index<K>: Send
where
    K: Key,
{
    /// HACK: `crossbeam-skiplist` doesn't seem to have an API for returning the old value.
    ///
    /// Whether the insert operation returns the old value or the new value.
    const INSERT_OLD: bool = true;

    type Handle: Handle<K>;
    fn new() -> Self;
    fn pin(&self) -> Self::Handle;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Key:
    arctic::Key + Clone + Hash + Eq + Send + Sync + Sized + ::concurrent_map::Minimum + 'static
{
    fn checksum(&self) -> u32;
}

impl Key for u64 {
    fn checksum(&self) -> u32 {
        *self as u32
    }
}

impl Key for String {
    fn checksum(&self) -> u32 {
        self.len() as u32
    }
}

pub trait Handle<K>: Send
where
    K: Key,
{
    fn get(&mut self, key: &K) -> Option<u32>;

    fn insert(&mut self, key: K, value: u32) -> Option<u32>;

    fn scan(&mut self, key: &K, count: usize) -> impl Iterator<Item = u32>;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub struct Arctic<K>(Arc<arctic::Map<K, u32>>);

impl<K> Index<K> for Arctic<K>
where
    K: Key,
{
    type Handle = Self;
    fn new() -> Self {
        Self(Arc::new(arctic::Map::default()))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::process(Arc::get_mut(&mut self.0).unwrap())).unwrap()
    }
}

impl<K> Handle<K> for Arctic<K>
where
    K: Key,
{
    fn get(&mut self, key: &K) -> Option<u32> {
        (*self.0).get(key)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        (*self.0).insert(&key, value)
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(arctic::stat::thread()).unwrap()
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        // FIXME
        core::iter::empty()
    }
}
