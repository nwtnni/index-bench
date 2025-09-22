//! Vendored from src/ds_impl/ebr
//! https://github.com/kaist-cp/smr-benchmark/tree/00b4e4d06d9b91f289b4644163f0cac0660d4af7

#[expect(clippy::needless_return)]
#[expect(clippy::too_many_arguments)]
#[expect(clippy::type_complexity)]
#[expect(dead_code)]
mod bonsai;

use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Bonsai<K: index::Key>(Arc<bonsai::BonsaiTreeMap<K, u32>>);

impl<K: index::Key, H: index::Hasher> Index<K, H> for Bonsai<K> {
    type Handle = Self;
    const IGNORE_INSERT: bool = true;

    fn new() -> Self {
        Self(Arc::new(bonsai::BonsaiTreeMap::new()))
    }

    fn pin(&self) -> Self::Handle {
        Self(self.0.clone())
    }
}

impl<K: index::Key> index::Handle<K> for Bonsai<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &crossbeam_ebr::pin();
        self.0.get(key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &crossbeam_ebr::pin();
        self.0.insert(key, value, guard);
        None
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
