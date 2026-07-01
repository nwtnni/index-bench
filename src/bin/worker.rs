use std::io::BufReader;
use std::io::BufWriter;

use index_bench::index;
use index_bench::index::Hasher;
use index_bench::workload::Key;
use index_bench::workload::KeyDistribution;
use index_bench::workload::Value;

fn main() -> anyhow::Result<()> {
    let mut stdin = BufReader::new(std::io::stdin().lock());
    let config: index_bench::Config = serde_json::from_reader(&mut stdin)?;
    let measurement = specialize_hash(config)?;
    let mut stdout = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut stdout, &measurement)?;
    Ok(())
}

fn specialize_hash(config: index_bench::Config) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.hash {
        index::Hash::RapidHash => specialize_value::<rapidhash::fast::RandomState>(config),
    }
}

fn specialize_value<H: Hasher>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.value {
        Value::U64 => specialize_key::<H, u64>(config),
        Value::Box => match config.index.name {
            index::Name::Arctic => specialize_key::<H, Box<u64>>(config),
            _ => unimplemented!("Box workload only implemented for Arctic"),
        },
    }
}

fn specialize_key<H: Hasher, V: index::Value + ::arctic::concurrent::Value + Send + Sync>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.key {
        Key::Ipv4 => specialize_index_u64::<H, index_bench::workload::Ipv4, V>(config),
        Key::U64 => specialize_index_u64::<H, index_bench::workload::U64, V>(config),
        Key::Snowflake => specialize_index_u64::<H, index_bench::workload::Snowflake, V>(config),
        Key::UuidV4 => specialize_index_u128::<H, index_bench::workload::UuidV4, V>(config),
        Key::Email => specialize_index_slice::<H, index_bench::workload::Email, V>(config),
        Key::Url => specialize_index_slice::<H, index_bench::workload::Url, V>(config),
    }
}

fn specialize_index_u64<
    H: Hasher,
    K: KeyDistribution<Key = u64>,
    V: index::Value + ::arctic::concurrent::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index::Name::Art => index_bench::run::<K, u64, art_sys::Rowex<K::Key>, H>(config),
        index::Name::Arctic => index_bench::run::<K, V, index::arctic::Map<K::Key, V>, H>(config),
        index::Name::Congee => index_bench::run::<K, u64, congee::Congee<usize, usize>, H>(config),
        index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, u64, crossbeam_skiplist::SkipMap<K::Key, u64>, H>(config)
        }
        index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::FbTree => index_bench::run::<K, u64, fbtree_sys::FbU64, H>(config),
        index::Name::Hot => index_bench::run::<K, u64, hot_sys::HotTreeU64, H>(config),
        index::Name::Papaya => {
            index_bench::run::<K, u64, papaya::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashIndex => {
            index_bench::run::<K, u64, scc::HashIndex<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashMap => {
            index_bench::run::<K, u64, scc::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccTreeIndex => {
            index_bench::run::<K, u64, scc::TreeIndex<K::Key, u64>, H>(config)
        }
        index::Name::Wormhole => index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config),
    }
}

fn specialize_index_u128<
    H: Hasher,
    K: KeyDistribution<Key = u128>,
    V: index::Value + ::arctic::concurrent::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index::Name::Art => index_bench::run::<K, u64, art_sys::Rowex<Vec<u8>>, H>(config),
        index::Name::Arctic => index_bench::run::<K, V, index::arctic::Map<K::Key, V>, H>(config),
        index::Name::Congee => unimplemented!(),
        index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, u64, crossbeam_skiplist::SkipMap<K::Key, u64>, H>(config)
        }
        index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::FbTree => index_bench::run::<K, u64, fbtree_sys::FbString, H>(config),
        index::Name::Hot => index_bench::run::<K, u64, hot_sys::HotTreeString, H>(config),
        index::Name::Papaya => {
            index_bench::run::<K, u64, papaya::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashIndex => {
            index_bench::run::<K, u64, scc::HashIndex<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashMap => {
            index_bench::run::<K, u64, scc::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccTreeIndex => {
            index_bench::run::<K, u64, scc::TreeIndex<K::Key, u64>, H>(config)
        }
        index::Name::Wormhole => index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config),
    }
}

#[allow(unused)]
fn specialize_index_slice<
    H: Hasher,
    K: KeyDistribution<Key = &'static [u8]>,
    V: index::Value + ::arctic::concurrent::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index::Name::Art => index_bench::run::<K, u64, art_sys::Rowex<Vec<u8>>, H>(config),
        index::Name::Arctic => index_bench::run::<
            K,
            V,
            index::arctic::Map<&'static ::arctic::key::Slice<::arctic::key::Terminated<b'\n'>>, V>,
            H,
        >(config),
        index::Name::Congee => unimplemented!(),
        index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, u64, crossbeam_skiplist::SkipMap<K::Key, u64>, H>(config)
        }
        index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::FbTree => index_bench::run::<K, u64, fbtree_sys::FbString, H>(config),
        index::Name::Hot => index_bench::run::<K, u64, hot_sys::HotTreeString, H>(config),
        index::Name::Papaya => {
            index_bench::run::<K, u64, papaya::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashIndex => {
            index_bench::run::<K, u64, scc::HashIndex<K::Key, u64, H>, H>(config)
        }
        index::Name::SccHashMap => {
            index_bench::run::<K, u64, scc::HashMap<K::Key, u64, H>, H>(config)
        }
        index::Name::SccTreeIndex => {
            index_bench::run::<K, u64, scc::TreeIndex<K::Key, u64>, H>(config)
        }
        index::Name::Wormhole => index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config),
    }
}

#[allow(unused)]
fn specialize_index_boxed_slice<
    H: Hasher,
    K: KeyDistribution<Key = &'static [u8]>,
    V: index::Value + ::arctic::concurrent::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index::Name::Art => index_bench::run::<K, u64, art_sys::Rowex<Vec<u8>>, H>(config),
        index::Name::Arctic => index_bench::run::<
            K,
            V,
            index::arctic::Map<::arctic::key::BoxedSlice<::arctic::key::Terminated<b'\n'>>, V>,
            H,
        >(config),
        index::Name::Congee => unimplemented!(),
        index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, u64, crossbeam_skiplist::SkipMap<Box<[u8]>, u64>, H>(config)
        }
        index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<Box<[u8]>, u64, H>, H>(config)
        }
        index::Name::FbTree => index_bench::run::<K, u64, fbtree_sys::FbString, H>(config),
        index::Name::Hot => index_bench::run::<K, u64, hot_sys::HotTreeString, H>(config),
        index::Name::Papaya => {
            index_bench::run::<K, u64, papaya::HashMap<Box<[u8]>, u64, H>, H>(config)
        }
        index::Name::SccHashIndex => {
            index_bench::run::<K, u64, scc::HashIndex<Box<[u8]>, u64, H>, H>(config)
        }
        index::Name::SccHashMap => {
            index_bench::run::<K, u64, scc::HashMap<Box<[u8]>, u64, H>, H>(config)
        }
        index::Name::SccTreeIndex => {
            index_bench::run::<K, u64, scc::TreeIndex<Box<[u8]>, u64>, H>(config)
        }
        index::Name::Wormhole => index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config),
    }
}
