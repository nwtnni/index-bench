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

pub trait Index: Send + Sync {
    type Handle: Handle;
    fn new() -> Self;
    fn pin(&self) -> Self::Handle;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub trait Handle {
    fn get(&mut self, key: u64) -> Option<u32>;

    fn insert(&mut self, key: u64, value: u32) -> Option<u32>;

    fn report(&mut self) -> serde_json::Value {
        serde_json::Value::Null
    }
}

pub struct Art(Arc<art::Map<u64, u32>>);

impl Index for Art {
    type Handle = Self;
    fn new() -> Self {
        Self(Arc::new(art::Map::default()))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }
}

impl Handle for Art {
    fn get(&mut self, key: u64) -> Option<u32> {
        (*self.0).get(key)
    }

    fn insert(&mut self, key: u64, value: u32) -> Option<u32> {
        (*self.0).insert(key, value)
    }
}
