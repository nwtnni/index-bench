use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map(Arc<scc::HashMap<u64, u32, rapidhash::fast::RandomState>>);

impl Index for Map {
    type Handle = Self;

    fn new() -> Self {
        Self(Arc::new(scc::HashMap::with_hasher(
            rapidhash::fast::RandomState::new(),
        )))
    }

    fn pin(&self) -> Self::Handle {
        Self(Arc::clone(&self.0))
    }
}

impl index::Handle for Map {
    fn get(&mut self, key: u64) -> Option<u32> {
        self.0.read_sync(&key, |_, value| *value)
    }

    fn insert(&mut self, key: u64, value: u32) -> Option<u32> {
        self.0.insert_sync(key, value).err().map(|(_, value)| value)
    }
}
