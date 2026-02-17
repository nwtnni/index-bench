use serde::Deserialize;
use serde::Serialize;

mod arctic;
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
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Index<K: Key, H> {
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

    type Send<'a>: IndexSend<K, H> + Send
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
}

pub trait IndexSend<K: Key, H> {
    type Handle<'a>: IndexPin<K>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a>;
}

pub trait Hasher: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}
impl<T> Hasher for T where T: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}

pub trait Key: ::arctic::Key {
    fn checksum(key: Self::Borrow<'_>) -> u64;
}

impl Key for u64 {
    fn checksum(key: Self::Borrow<'_>) -> u64 {
        key
    }
}

impl Key for Vec<u8> {
    fn checksum(key: Self::Borrow<'_>) -> u64 {
        key.len() as u64
    }
}

pub trait IndexPin<K: Key> {
    fn enable_membarrier(&self) {}

    fn get(&mut self, key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64>;

    fn insert(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64>;

    fn update(
        &mut self,
        key: <K as ::arctic::raw::Key>::Borrow<'static>,
        value: u64,
    ) -> Option<u64> {
        self.insert(key, value)
    }

    fn remove(&mut self, _key: <K as ::arctic::raw::Key>::Borrow<'static>) -> Option<u64> {
        unimplemented!(
            "TODO: implement remove for {}",
            std::any::type_name::<Self>()
        )
    }

    fn scan(
        &mut self,
        _key: <K as ::arctic::raw::Key>::Borrow<'static>,
        _count: usize,
        _buffer: &mut Vec<u64>,
    ) {
        unimplemented!("TODO: implement scan for {}", std::any::type_name::<Self>())
    }

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
