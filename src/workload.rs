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

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    U64,
    Email,
    Prefix(usize),
    Sparse(f64),
}

static ACKNOWLEDGED: Acknowledged = Acknowledged::new();

impl Config {
    pub(crate) fn operation_count_per_thread(&self, thread_count: usize) -> usize {
        (match self.load {
            true => self.ycsb.record_count,
            false => self.ycsb.operation_count,
        }) / thread_count
    }

    pub(crate) fn loader<K: KeyDistribution>(
        &self,
        config: Key,
        thread_count: usize,
        thread_id: usize,
    ) -> Loader<K> {
        Loader {
            inner: self.ycsb.loader(thread_count, thread_id),
            keys: K::new(config),
        }
    }

    pub(crate) fn runner<K: KeyDistribution>(&self, config: Key) -> Runner<K> {
        Runner {
            inner: self.ycsb.runner(&ACKNOWLEDGED),
            keys: K::new(config),
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

pub struct Runner<'ycsb, K> {
    inner: ycsb::Runner<'ycsb>,
    keys: K,
}

impl<'ycsb, K: KeyDistribution> Runner<'ycsb, K> {
    pub(crate) fn next_operation<R: rand::Rng>(&mut self, rng: &mut R) -> ycsb::Operation {
        self.inner.next_operation(rng)
    }

    pub(crate) fn next_key_range<R: rand::Rng>(&mut self, rng: &mut R, start: ycsb::Key) -> K::Key {
        let delta = self.inner.next_scan_length(rng);
        let end = start.id() + delta as u64;
        self.keys.get(end)
    }

    pub(crate) fn next_key_read<R: rand::Rng>(&mut self, rng: &mut R) -> (ycsb::Key, K::Key) {
        let key = self.inner.next_key_read(rng);
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn next_key_insert(&mut self) -> (ycsb::Key, K::Key) {
        let key = self.inner.next_key_insert();
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn acknowledge(&mut self, key: ycsb::Key) {
        self.inner.acknowledge(key)
    }
}

pub trait KeyDistribution {
    type Key: index::Key;
    fn new(config: Key) -> Self;
    fn get(&self, index: u64) -> Self::Key;
}

pub struct U64;

impl KeyDistribution for U64 {
    type Key = u64;

    fn new(_: Key) -> Self {
        Self
    }

    fn get(&self, index: u64) -> Self::Key {
        index
    }
}

static EMAIL_BUFFER: LazyLock<String> = LazyLock::new(|| {
    std::fs::read_to_string("data/email.txt").expect("Failed to find data/email.txt")
});

static EMAIL: LazyLock<Vec<&'static str>> = LazyLock::new(|| EMAIL_BUFFER.lines().collect());

pub struct Email(&'static [&'static str]);

impl KeyDistribution for Email {
    type Key = String;

    fn new(_: Key) -> Self {
        Self(LazyLock::force(&EMAIL).as_slice())
    }

    fn get(&self, index: u64) -> Self::Key {
        self.0[index as usize % self.0.len()].to_owned()
    }
}

pub struct Prefix(usize);

impl KeyDistribution for Prefix {
    type Key = Vec<u8>;

    fn new(config: Key) -> Self {
        let len = match config {
            Key::Prefix(len) => len,
            _ => unreachable!(),
        };

        Self(len)
    }

    fn get(&self, index: u64) -> Self::Key {
        let mut key = Vec::with_capacity(self.0 + 8);
        key.extend((0..self.0).map(|_| 0));
        key.extend(index.to_be_bytes());
        key
    }
}

pub struct Sparse(f64);

impl KeyDistribution for Sparse {
    type Key = u64;

    fn new(config: Key) -> Self {
        let sparse = match config {
            Key::Sparse(sparse) => sparse,
            _ => unreachable!(),
        };

        Self(sparse)
    }

    fn get(&self, index: u64) -> Self::Key {
        (index as f64 * self.0) as u64
    }
}
