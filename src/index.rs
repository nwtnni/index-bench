use core::borrow::Borrow as _;

use serde::Deserialize;
use serde::Serialize;

pub mod arctic;
mod art;
// mod b_plus_tree;
// mod bz_tree;
// mod concurrent_map;
// mod congee;
// mod contrie;
// mod crossbeam_skiplist;
mod dash_map;
mod fb_tree;
// pub mod kaist;
// mod papaya;
// mod scc;
mod hot;
mod wormhole;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub struct Config {
    pub hash: Hash,
    pub name: Name,
    #[serde(default)]
    pub retry_scan: usize,
    #[serde(default = "reclaim_threshold")]
    pub reclaim_threshold: usize,
    #[serde(default = "smr")]
    pub smr: Smr,
    #[serde(default = "membarrier")]
    pub membarrier: bool,
}

fn reclaim_threshold() -> usize {
    64
}

fn smr() -> Smr {
    if cfg!(feature = "smr-disable") {
        Smr::Disable
    } else if cfg!(feature = "smr-epoch") {
        Smr::Epoch
    } else if cfg!(feature = "smr-seize") {
        Smr::Seize
    } else {
        Smr::Hazard
    }
}

fn membarrier() -> bool {
    cfg!(feature = "membarrier")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hash {
    RapidHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Smr {
    Disable,
    Epoch,
    Seize,
    Hazard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Name {
    Art,
    Arctic,
    // Bonsai,
    // BPlusTree,
    // BzTree,
    // ConcurrentMap,
    // Congee,
    // Contrie,
    // CrossbeamSkiplist,
    DashMap,
    FbTree,
    // Papaya,
    // SccHashMap,
    // SccTreeIndex,
    Wormhole,
    Hot,
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Index<K: Key, V: Value, H> {
    /// HACK
    ///
    /// - `crossbeam-skiplist` returns the new value instead of the old
    /// - `kaist::bonsai` returns whether the insertion succeeded
    ///
    /// Whether to skip validation of `insert`.
    const IGNORE_INSERT: bool = false;
    const IGNORE_UPDATE: bool = Self::IGNORE_INSERT;
    /// - `crossbeam-skiplist` can see a removal during insertion: https://github.com/crossbeam-rs/crossbeam/issues/1023
    const IGNORE_GET: bool = false;

    type Send<'a>: IndexSend<K, V, H> + Send
    where
        Self: 'a;

    fn new(config: &Config) -> Self;

    fn send<'a>(&'a self) -> Self::Send<'a>;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }

    // Report the total size of the keys and values in this map
    fn memory_key_value(&mut self) -> u64 {
        0
    }

    // Report the maximum number of unreclaimed allocations
    fn garbage(&mut self) -> u32 {
        0
    }
}

pub trait IndexSend<K: Key, V: Value, H> {
    type Handle<'a>: IndexPin<K, V>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a>;
}

pub trait Hasher: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}
impl<T> Hasher for T where T: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}

pub trait Key {
    type Borrow: Copy;
    fn checksum(key: <Self as Key>::Borrow) -> u64;
}

impl Key for u64 {
    type Borrow = Self;
    fn checksum(key: <Self as Key>::Borrow) -> u64 {
        *key.borrow()
    }
}

impl Key for u128 {
    type Borrow = Self;
    fn checksum(key: <Self as Key>::Borrow) -> u64 {
        *key.borrow() as u64
    }
}

impl Key for Vec<u8> {
    type Borrow = &'static [u8];
    fn checksum(key: <Self as Key>::Borrow) -> u64 {
        key.len() as u64
    }
}

pub trait Value: ::arctic::Value {
    fn from_checksum(checksum: u64) -> Self;

    fn from_borrow<'a>(borrow: &'a <Self as ::arctic::concurrent::Value>::Target) -> Self
    where
        Self: 'a;
}

impl Value for u64 {
    fn from_checksum(checksum: u64) -> Self {
        checksum
    }

    fn from_borrow<'a>(borrow: &u64) -> Self
    where
        Self: 'a,
    {
        *borrow
    }
}

impl Value for Box<u64> {
    fn from_checksum(checksum: u64) -> Self {
        Box::new(checksum)
    }

    // Uhhh. This sucks. Needed for scans on dynamically allocated values,
    // but using `Arc` in that case would certainly be much better...
    fn from_borrow<'a>(borrow: &'a u64) -> Self
    where
        Self: 'a,
    {
        Box::new(*borrow)
    }
}

pub trait IndexPin<K: Key, V: Value> {
    fn enable_membarrier(&self) {}

    fn get(&mut self, key: <K as Key>::Borrow) -> Option<V>;

    fn insert(&mut self, key: <K as Key>::Borrow, value: V) -> Option<V>;

    fn update(&mut self, key: <K as Key>::Borrow, value: V) -> Option<V> {
        self.insert(key, value)
    }

    fn remove(&mut self, _key: <K as Key>::Borrow) -> Option<V> {
        unimplemented!(
            "TODO: implement remove for {}",
            std::any::type_name::<Self>()
        )
    }

    fn scan(&mut self, _key: <K as Key>::Borrow, _count: usize, _buffer: &mut Vec<V>) {
        unimplemented!("TODO: implement scan for {}", std::any::type_name::<Self>())
    }

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
