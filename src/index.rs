use core::hash::Hash;
use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;

pub mod scc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum Config {
    Art,
    Scc,
}

pub trait Index<K>: Send + Sync
where
    K: Key,
{
    type Handle: Handle<K>;
    fn new() -> Self;
    fn pin(&self) -> Self::Handle;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Key: art::Key + Hash + Eq + Send + Sync + Sized {
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

pub trait Handle<K>
where
    K: Key,
{
    fn get(&mut self, key: &K) -> Option<u32>;

    fn insert(&mut self, key: K, value: u32) -> Option<u32>;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub struct Art<K>(Arc<art::Map<K, u32>>);

impl<K> Index<K> for Art<K>
where
    K: Key,
{
    type Handle = Self;
    fn new() -> Self {
        Self(Arc::new(art::Map::default()))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }

    #[cfg(feature = "stat")]
    fn report(&mut self) -> serde_json::Value {
        serde_json::to_value(art::stat::process(Arc::get_mut(&mut self.0).unwrap())).unwrap()
    }
}

impl<K> Handle<K> for Art<K>
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
        serde_json::to_value(art::stat::thread()).unwrap()
    }
}
