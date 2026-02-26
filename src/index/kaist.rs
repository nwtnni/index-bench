//! Vendored from src/ds_impl/ebr
//! https://github.com/kaist-cp/smr-benchmark/tree/00b4e4d06d9b91f289b4644163f0cac0660d4af7

#[expect(clippy::needless_return)]
#[expect(clippy::too_many_arguments)]
#[expect(clippy::type_complexity)]
mod bonsai;

use crate::Index;
use crate::index;

pub use bonsai::BonsaiTreeMap;

impl<K: index::Key, H: index::Hasher> Index<K, u64, H> for bonsai::BonsaiTreeMap<K, u64> {
    type Send<'a> = &'a Self;
    const IGNORE_INSERT: bool = true;

    fn new(_: &index::Config) -> Self {
        bonsai::BonsaiTreeMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, u64, H>
    for &'_ bonsai::BonsaiTreeMap<K, u64>
{
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K, u64> for &'_ bonsai::BonsaiTreeMap<K, u64> {
    fn get(&mut self, key: &K) -> Option<u64> {
        let guard = &crossbeam_ebr::pin();
        bonsai::BonsaiTreeMap::get(self, key, guard).copied()
    }

    fn insert(&mut self, key: K, value: u64) -> Option<u64> {
        let guard = &crossbeam_ebr::pin();
        bonsai::BonsaiTreeMap::insert(self, key, value, guard);
        None
    }
}
