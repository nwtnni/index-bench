use core::marker::PhantomData;

use cartesian::Cartesian;
use serde::Deserialize;
use serde::Serialize;

use crate::index;

#[derive(Cartesian)]
#[rustfmt::skip]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub load: bool,

    pub key: Key,

    #[cartesian(flatten)]
    #[serde(flatten)]
    ycsb: ycsb::Workload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Key {
    U64,
}

impl Config {
    pub(crate) fn operation_count(&self, thread_count: usize) -> usize {
        self.ycsb.record_count() / thread_count
    }

    pub(crate) fn loader<K: index::Key>(&self, thread_count: usize, thread_id: usize) -> Loader<K> {
        Loader {
            inner: self.ycsb.loader(thread_count, thread_id),
            _key: PhantomData,
        }
    }
}

pub struct Loader<K> {
    inner: ycsb::Loader,
    _key: PhantomData<K>,
}

impl<K> Loader<K>
where
    K: index::Key,
{
    #[inline]
    pub(crate) fn next_key(&mut self) -> Option<K> {
        Some(K::from_index(self.inner.next_key()?.id()))
    }
}
