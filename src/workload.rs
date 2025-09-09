use std::sync::LazyLock;

use cartesian::Cartesian;
use serde::Deserialize;
use serde::Serialize;
use ycsb::Acknowledged;

use crate::index;

#[derive(Cartesian)]
#[rustfmt::skip]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub load: bool,

    pub key: Key,

    #[cartesian(flatten)]
    #[serde(flatten)]
    pub ycsb: ycsb::Workload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    U64,
    Email,
}

static ACKNOWLEDGED: Acknowledged = Acknowledged::new();

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

    pub(crate) fn runner<K: KeyDistribution>(&self) -> Runner<K> {
        Runner {
            inner: self.ycsb.runner(&ACKNOWLEDGED),
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

pub struct Runner<K> {
    inner: ycsb::Runner<'static>,
    keys: K,
}

impl<K: KeyDistribution> Runner<K> {
    pub(crate) fn next_operation<R: rand::Rng>(&mut self, rng: &mut R) -> ycsb::Operation {
        self.inner.next_operation(rng)
    }

    pub(crate) fn next_key_read<R: rand::Rng>(&mut self, rng: &mut R) -> (ycsb::Key, K::Key) {
        let key = self.inner.next_key_read(rng);
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn next_key_insert<R: rand::Rng>(
        &mut self,
        rng: &mut R,
        window: u64,
    ) -> (ycsb::Key, K::Key) {
        let key = self.inner.next_key_insert(rng, window);
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn acknowledge(&mut self, key: ycsb::Key) {
        self.inner.acknowledge(key)
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
