use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

pub mod bz_tree;
pub mod concurrent_map;
pub mod crossbeam_skiplist;
pub mod kaist;
pub mod papaya;
pub mod scc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub struct Config {
    pub hash: Hash,
    pub name: Name,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hash {
    RapidHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Name {
    Arctic,
    Bonsai,
    BzTree,
    ConcurrentMap,
    CrossbeamSkiplist,
    Papaya,
    Scc,
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Index<K, H>: Send
where
    K: Key,
    H: Hasher,
{
    /// HACK
    ///
    /// - `crossbeam-skiplist` returns the new value instead of the old
    /// - `kaist::bonsai` returns whether the insertion succeeded
    ///
    /// Whether the insert operation returns the old value or the new value.
    const IGNORE_INSERT: bool = false;

    type Handle: Handle<K>;
    fn new() -> Self;
    fn pin(&self) -> Self::Handle;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Hasher: core::hash::BuildHasher + Default + Send + Sync {}
impl<T> Hasher for T where T: core::hash::BuildHasher + Default + Send + Sync {}

pub trait Key:
    arctic::Key
    + Clone
    + core::hash::Hash
    + Eq
    + Send
    + Sync
    + Sized
    + ::concurrent_map::Minimum
    + 'static
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

impl Key for Vec<u8> {
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

impl<K, H> Index<K, H> for Arctic<K>
where
    K: Key,
    H: Hasher,
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
