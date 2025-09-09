use std::sync::LazyLock;

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
    Email,
}

impl Config {
    pub(crate) fn operation_count(&self, thread_count: usize) -> usize {
        self.ycsb.record_count() / thread_count
    }

    pub(crate) fn loader<K: KeyDistribution>(
        &self,
        thread_count: usize,
        thread_id: usize,
    ) -> Loader<K> {
        Loader {
            inner: self.ycsb.loader(thread_count, thread_id),
            keys: K::default(),
        }
    }
}

pub struct Loader<K> {
    inner: ycsb::Loader,
    keys: K,
}

impl<K> Loader<K>
where
    K: KeyDistribution,
{
    #[inline]
    pub(crate) fn next_key(&mut self) -> Option<K::Key> {
        Some(self.keys.get(self.inner.next_key()?.id()))
    }
}

pub trait KeyDistribution: Default {
    type Key: index::Key;
    fn get(&self, index: u64) -> Self::Key;
}

#[derive(Default)]
pub struct U64;

impl KeyDistribution for U64 {
    type Key = u64;
    fn get(&self, index: u64) -> Self::Key {
        index
    }
}

static EMAILS: LazyLock<String> = LazyLock::new(|| {
    std::fs::read_to_string("data/email.txt").expect("Failed to find data/email.txt")
});

pub struct Email(Vec<&'static str>);

impl Default for Email {
    fn default() -> Self {
        Self(EMAILS.lines().collect())
    }
}

impl KeyDistribution for Email {
    type Key = String;
    fn get(&self, index: u64) -> Self::Key {
        self.0[index as usize % self.0.len()].to_owned()
    }
}
