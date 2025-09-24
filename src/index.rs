use serde::Deserialize;
use serde::Serialize;

mod arctic;
mod b_plus_tree;
mod bz_tree;
pub mod concurrent_map;
pub mod congee;
mod contrie;
mod crossbeam_skiplist;
mod dash_map;
pub mod kaist;
mod papaya;
mod scc;

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
    BPlusTree,
    BzTree,
    ConcurrentMap,
    Congee,
    Contrie,
    CrossbeamSkiplist,
    DashMap,
    Papaya,
    Scc,
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Index<K, H>: Send + Sync
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

    type Handle<'a>: Handle<K>
    where
        Self: 'a;
    fn new() -> Self;
    fn pin<'a>(&'a self) -> Self::Handle<'a>;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Hasher: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}
impl<T> Hasher for T where T: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}

pub trait Key:
    ::arctic::Key
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

pub trait Handle<K>
where
    K: Key,
{
    fn get(&mut self, key: &K) -> Option<u32>;

    fn insert(&mut self, key: K, value: u32) -> Option<u32>;

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
