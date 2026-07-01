use serde::Deserialize;
use serde::Serialize;

pub mod concurrent;
pub mod sequential;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub struct Config {
    #[serde(default)]
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
    256
}

fn smr() -> Smr {
    if cfg!(feature = "smr-disable") {
        Smr::Disable
    } else if cfg!(feature = "smr-epoch") {
        Smr::Epoch
    } else if cfg!(feature = "smr-hazard") {
        Smr::Hazard
    } else {
        Smr::Seize
    }
}

fn membarrier() -> bool {
    cfg!(feature = "membarrier")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hash {
    #[default]
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
    ArcticSeq,
    ConcurrentMap,
    Congee,
    Contrie,
    CrossbeamSkiplist,
    DashMap,
    FbTree,
    Leapfrog,
    Masstree,
    Papaya,
    SccHashIndex,
    SccHashMap,
    SccTreeIndex,
    StdBTreeMap,
    StdHashMap,
    Wormhole,
    Hot,
}

pub enum Insert {
    Old,
    New,
    OldExists,
}

pub trait Hasher: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}
impl<T> Hasher for T where T: core::hash::BuildHasher + Clone + Default + Send + Sync + 'static {}

pub trait Index<K, V, H> {
    /// Whether this index can be accessed from more than one thread
    const CONCURRENT: bool = true;

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

pub trait IndexSend<K, V, H> {
    type Handle<'a>: IndexPin<K, V>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a>;
}

pub trait IndexPin<K, V> {
    fn enable_membarrier(&self) {}

    fn get(&mut self, key: K);

    fn insert(&mut self, key: K, value: V);

    fn update(&mut self, key: K, value: V) {
        self.insert(key, value)
    }

    fn remove(&mut self, _key: K) {
        unimplemented!(
            "TODO: implement remove for {}",
            std::any::type_name::<Self>()
        )
    }

    fn scan(&mut self, _key: K, _count: usize) {
        unimplemented!("TODO: implement scan for {}", std::any::type_name::<Self>())
    }

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Key {
    fn with_slice<F, T>(&self, with: F) -> T
    where
        F: FnOnce(&[u8]) -> T;
}

impl Key for u64 {
    fn with_slice<F, T>(&self, with: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        with(&self.to_ne_bytes())
    }
}

impl Key for u128 {
    fn with_slice<F, T>(&self, with: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        with(&self.to_ne_bytes())
    }
}

impl Key for &'static [u8] {
    fn with_slice<F, T>(&self, with: F) -> T
    where
        F: FnOnce(&[u8]) -> T,
    {
        with(self)
    }
}

pub trait Value {
    fn checksum(key: &[u8]) -> Self;
}

impl Value for u64 {
    fn checksum(key: &[u8]) -> Self {
        let mut buffer = [0; 8];
        buffer.copy_from_slice(key);
        Self::from_ne_bytes(buffer)
    }
}

impl<V> Value for Box<V>
where
    V: Value,
{
    fn checksum(key: &[u8]) -> Self {
        Box::new(V::checksum(key))
    }
}
