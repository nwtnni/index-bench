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
    Url,
    // Prefix(usize),
    Sparse(f64),
    Kmer,
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
    pub(crate) fn next_key(&mut self) -> Option<<K::Key as ::arctic::raw::Key>::Borrow<'static>> {
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
    ) -> <K::Key as ::arctic::raw::Key>::Borrow<'static> {
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
    ) -> (ycsb::Key, <K::Key as ::arctic::raw::Key>::Borrow<'static>) {
        let key = self.inner.next_key_read(rng);
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn next_key_insert(
        &mut self,
    ) -> (ycsb::Key, <K::Key as ::arctic::raw::Key>::Borrow<'static>) {
        let key = self.inner.next_key_insert();
        (key, self.keys.get(key.id()))
    }

    pub(crate) fn acknowledge(&mut self, key: ycsb::Key) {
        self.inner.acknowledge(key)
    }
}

pub trait KeyDistribution {
    type Key: index::Key;
    fn new(config: &Key) -> Self;
    fn get(&self, index: u64) -> <Self::Key as ::arctic::raw::Key>::Borrow<'static>;
}

pub struct U64;

impl KeyDistribution for U64 {
    type Key = u64;

    fn new(_: &Key) -> Self {
        Self
    }

    fn get(&self, index: u64) -> Self::Key {
        index
    }
}

static EMAIL_BUFFER: LazyLock<String> = LazyLock::new(|| {
    std::fs::read_to_string("data/email.txt").expect("Failed to find data/email.txt")
});

static EMAIL_INDEX: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| EMAIL_BUFFER.split_inclusive('\n').collect());

pub struct Email(&'static [&'static str]);

impl KeyDistribution for Email {
    type Key = String;

    fn new(_: &Key) -> Self {
        Self(LazyLock::force(&EMAIL_INDEX).as_slice())
    }

    fn get(&self, index: u64) -> &'static str {
        self.0[index as usize % self.0.len()]
    }
}

static URL_BUFFER: LazyLock<String> =
    LazyLock::new(|| std::fs::read_to_string("data/url.txt").expect("Failed to find data/url.txt"));

static URL_INDEX: LazyLock<Vec<&'static str>> =
    LazyLock::new(|| URL_BUFFER.split_inclusive('\n').collect());

pub struct Url(&'static [&'static str]);

impl KeyDistribution for Url {
    type Key = String;

    fn new(_: &Key) -> Self {
        Self(LazyLock::force(&URL_INDEX).as_slice())
    }

    fn get(&self, index: u64) -> &'static str {
        self.0[index as usize % self.0.len()]
    }
}

// pub struct Prefix(usize);
//
// impl KeyDistribution for Prefix {
//     type Key = Vec<u8>;
//
//     fn new(config: &Key) -> Self {
//         let len = match config {
//             Key::Prefix(len) => len,
//             _ => unreachable!(),
//         };
//
//         Self(*len)
//     }
//
//     fn get(&self, index: u64) -> Self::Key {
//         let mut key = Vec::with_capacity(self.0 + 8);
//         key.extend((0..self.0).map(|_| 0));
//         key.extend(index.to_be_bytes());
//         key
//     }
// }

pub struct Sparse(f64);

impl KeyDistribution for Sparse {
    type Key = u64;

    fn new(config: &Key) -> Self {
        let sparse = match config {
            Key::Sparse(sparse) => sparse,
            _ => unreachable!(),
        };

        Self(*sparse)
    }

    fn get(&self, index: u64) -> Self::Key {
        (index as f64 * self.0) as u64
    }
}

static KMER_BUFFER: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::fs::read("data/SRR31218470.bin").expect("Failed to find data/SRR31218470.bin.txt")
});

pub struct Kmer(&'static [u8]);

impl KeyDistribution for Kmer {
    type Key = u64;

    fn new(_: &Key) -> Self {
        Self(LazyLock::force(&KMER_BUFFER).as_ref())
    }

    /// https://github.com/nicolasgarza/fetchkmer/blob/d910161007cb7e4c3396a49998081db6d6a1f134/src/fetch.rs#L1-L20
    fn get(&self, index: u64) -> Self::Key {
        // Hard-code for 28-mer (from KMC-2 and Gerbil papers) and f. vesca sequence
        // Each sequence is 151 base pairs = 302 bits = 38 bytes
        // There are 123 28-mers in each sequence (0..28, 1..29, ..., 123..151)
        let sequence_index = index / 123;
        let sequence_byte_offset = sequence_index * 38;
        let kmer_index = index % 123;

        let bit_offset = kmer_index * 2;
        let byte_offset = bit_offset / 8;
        let intra_byte_offset = bit_offset % 8;

        // let mut buf: u128 = 0;
        // for j in 0..K / 2 {
        //     let byte = self.0[byte_offset as usize + j];
        //     buf |= (byte as u128) << (8 * j);
        // }
        //
        // let shifted = buf >> intra_byte_offset;
        //
        // // mask exactly 64 bits
        // (shifted & 0xFFFF_FFFF_FFFF_FFFFu128) as u64

        // Load as native endian integer
        // FIXME: doesn't check for boundary condition near EOF
        let buf = u128::from_be_bytes(
            self.0[(sequence_byte_offset + byte_offset) as usize..][..16]
                .try_into()
                .unwrap(),
        );

        // 28-mer means keep top 56 bits
        const K_MASK: u64 = !(u64::MAX >> 56);
        (((buf << intra_byte_offset) >> 64) as u64) & K_MASK
    }
}
