use core::hash::Hasher;
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

    pub value: Value,

    #[cartesian(flatten)]
    #[serde(flatten)]
    pub ycsb: ycsb::Workload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Ipv4,
    U64,
    Snowflake,
    UuidV4,
    Email,
    Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    U64,
    Box,
}

impl Config {
    pub(crate) fn operation_count_per_thread(&self, thread_count: usize) -> usize {
        (if self.load {
            self.ycsb.record_count
        } else {
            self.ycsb.operation_count
        }) / thread_count
    }

    pub(crate) fn loader<K: KeyDistribution>(
        &self,
        config: &Key,
        thread_count: usize,
        thread_id: usize,
    ) -> Loader<K> {
        Loader {
            inner: self.ycsb.loader(thread_count, thread_id),
            keys: K::new(config),
        }
    }

    pub(crate) fn runner<K: KeyDistribution>(&self, config: &Key) -> Runner<K> {
        Runner {
            inner: self.ycsb.runner(),
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
    pub(crate) fn next_key(&mut self) -> Option<<K::Key as index::Key>::Borrow> {
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

    #[expect(unused)]
    pub(crate) fn next_key_range<R: rand::Rng>(
        &mut self,
        rng: &mut R,
        start: ycsb::Key,
    ) -> <K::Key as index::Key>::Borrow {
        let delta = self.inner.next_scan_length(rng);
        let end = start.id() + delta as u64 - 1;
        self.keys.get(end)
    }

    pub(crate) fn next_scan_length<R: rand::Rng>(&mut self, rng: &mut R) -> usize {
        self.inner.next_scan_length(rng)
    }

    pub(crate) fn next_key_read<R: rand::Rng>(
        &mut self,
        rng: &mut R,
    ) -> (ycsb::Key, <K::Key as index::Key>::Borrow) {
        let key = self.inner.next_key_read(rng);
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn next_key_insert(&mut self) -> <K::Key as index::Key>::Borrow {
        let key = self.inner.next_key_insert();
        self.keys.get(key.id())
    }
}

pub trait KeyDistribution {
    type Key: index::Key;
    fn new(config: &Key) -> Self;
    fn get(&self, index: u64) -> <Self::Key as index::Key>::Borrow;
}

static IP_BUFFER: LazyLock<Vec<u8>> =
    LazyLock::new(|| std::fs::read("data/ipv4.bin").expect("Failed to find data/ipv4.bin"));

pub struct Ipv4;

impl KeyDistribution for Ipv4 {
    type Key = u64;

    fn new(_: &Key) -> Self {
        LazyLock::force(&IP_BUFFER);
        Self
    }

    fn get(&self, index: u64) -> u64 {
        let index = index as usize % (IP_BUFFER.len() / 4);
        let data = IP_BUFFER[index..].first_chunk::<4>().unwrap();
        u32::from_le_bytes(*data) as u64
    }
}

pub struct U64;

impl KeyDistribution for U64 {
    type Key = u64;

    fn new(_: &Key) -> Self {
        Self
    }

    fn get(&self, index: u64) -> u64 {
        index
    }
}

static SNOWFLAKE_BUFFER: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::fs::read("data/snowflake.bin").expect("Failed to find data/snowflake.bin")
});

pub struct Snowflake;

impl KeyDistribution for Snowflake {
    type Key = u64;

    fn new(_: &Key) -> Self {
        LazyLock::force(&SNOWFLAKE_BUFFER);
        Self
    }

    fn get(&self, index: u64) -> u64 {
        let index = index as usize % (SNOWFLAKE_BUFFER.len() / 8);
        let data = SNOWFLAKE_BUFFER[index..].first_chunk::<8>().unwrap();
        u64::from_le_bytes(*data)
    }
}

pub struct UuidV4;

impl KeyDistribution for UuidV4 {
    type Key = u128;

    fn new(_: &Key) -> Self {
        Self
    }

    fn get(&self, index: u64) -> u128 {
        let mut hasher = rapidhash::fast::RapidHasher::default();
        hasher.write_u64(index);
        let lo = hasher.finish();
        hasher.write_u64(index);
        let hi = hasher.finish();

        uuid::Builder::from_random_bytes(((lo as u128) | ((hi as u128) << 64)).to_ne_bytes())
            .with_version(uuid::Version::Random)
            .with_variant(uuid::Variant::RFC4122)
            .into_uuid()
            .as_u128()
    }
}

static EMAIL_BUFFER: LazyLock<String> = LazyLock::new(|| {
    std::fs::read_to_string("data/email.txt").expect("Failed to find data/email.txt")
});

static EMAIL_INDEX: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| EMAIL_BUFFER.split_inclusive('\n').collect());

pub struct Email(&'static [&'static str]);

impl KeyDistribution for Email {
    type Key = Vec<u8>;

    fn new(_: &Key) -> Self {
        Self(LazyLock::force(&EMAIL_INDEX).as_slice())
    }

    fn get(&self, index: u64) -> &'static [u8] {
        self.0[index as usize % self.0.len()].as_bytes()
    }
}

static URL_BUFFER: LazyLock<String> =
    LazyLock::new(|| std::fs::read_to_string("data/url.txt").expect("Failed to find data/url.txt"));

static URL_INDEX: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| URL_BUFFER.split_inclusive('\n').collect());

pub struct Url(&'static [&'static str]);

impl KeyDistribution for Url {
    type Key = Vec<u8>;

    fn new(_: &Key) -> Self {
        Self(LazyLock::force(&URL_INDEX).as_slice())
    }

    fn get(&self, index: u64) -> &'static [u8] {
        self.0[index as usize % self.0.len()].as_bytes()
    }
}
