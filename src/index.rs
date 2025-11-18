use serde::Deserialize;
use serde::Serialize;

mod arctic;
mod art;
mod b_plus_tree;
mod bz_tree;
mod concurrent_map;
mod congee;
mod contrie;
mod crossbeam_skiplist;
mod dash_map;
mod fb_tree;
pub mod kaist;
mod papaya;
mod scc;
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
    Hazard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Name {
    Art,
    Arctic,
    Bonsai,
    BPlusTree,
    BzTree,
    ConcurrentMap,
    Congee,
    Contrie,
    CrossbeamSkiplist,
    DashMap,
    FbTree,
    Papaya,
    SccHashMap,
    SccTreeIndex,
    Wormhole,
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Index<K, H>
where
    K: Key,
    H: Hasher,
{
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
}

pub trait IndexSend<K, H>
where
    K: Key,
    H: Hasher,
{
    type Handle<'a>: IndexPin<K>
    where
        Self: 'a;
    fn pin<'a>(&'a self) -> Self::Handle<'a>;
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

pub trait IndexPin<K>
where
    K: Key,
{
    fn get(&mut self, key: &K) -> Option<u32>;

    fn insert(&mut self, key: K, value: u32) -> Option<u32>;

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        self.insert(key, value)
    }

    fn remove(&mut self, _key: K) -> Option<u32> {
        unimplemented!(
            "TODO: implement remove for {}",
            std::any::type_name::<Self>()
        )
    }

    fn range<'a>(
        &'a mut self,
        _retry_scan: usize,
        _min: &'a K,
        _max: &'a K,
        _output: &mut Vec<(K, u32)>,
    ) {
        unimplemented!(
            "TODO: implement range for {}",
            std::any::type_name::<Self>()
        );
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        unimplemented!("TODO: implement scan for {}", std::any::type_name::<Self>());

        #[expect(unreachable_code)]
        core::iter::empty()
    }

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
