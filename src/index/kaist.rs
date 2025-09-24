//! Vendored from src/ds_impl/ebr
//! https://github.com/kaist-cp/smr-benchmark/tree/00b4e4d06d9b91f289b4644163f0cac0660d4af7

#[expect(clippy::needless_return)]
#[expect(clippy::too_many_arguments)]
#[expect(clippy::type_complexity)]
mod bonsai;

use crate::Index;
use crate::index;

pub use bonsai::BonsaiTreeMap;

impl<K: index::Key, H: index::Hasher> Index<K, H> for bonsai::BonsaiTreeMap<K, u32> {
    type Handle<'a> = &'a Self;
    const IGNORE_INSERT: bool = true;

    fn new() -> Self {
        bonsai::BonsaiTreeMap::new()
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::Handle<K> for &'_ bonsai::BonsaiTreeMap<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &crossbeam_ebr::pin();
        bonsai::BonsaiTreeMap::get(self, key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &crossbeam_ebr::pin();
        bonsai::BonsaiTreeMap::insert(self, key, value, guard);
        None
    }
}
