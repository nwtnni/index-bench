use std::io::BufReader;
use std::io::BufWriter;

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
        index_bench::index::Hash::RapidHash => {
            specialize_value::<rapidhash::fast::RandomState>(config)
        }
    }
}

fn specialize_value<H: index_bench::index::Hasher>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.value {
        index_bench::workload::Value::U64 => specialize_key::<H, u64>(config),
        index_bench::workload::Value::Box => match config.index.name {
            index_bench::index::Name::Arctic => specialize_key::<H, Box<u64>>(config),
            _ => unimplemented!("Box workload only implemented for Arctic"),
        },
    }
}

fn specialize_key<
    H: index_bench::index::Hasher,
    V: index_bench::index::Value + ::arctic::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.key {
        index_bench::workload::Key::U64 => {
            specialize_index_u64::<H, index_bench::workload::U64, V>(config)
        }
        index_bench::workload::Key::Sparse(_) => {
            specialize_index_u64::<H, index_bench::workload::Sparse, V>(config)
        }
        index_bench::workload::Key::Kmer => {
            specialize_index_u64::<H, index_bench::workload::Kmer, V>(config)
        }
        index_bench::workload::Key::Ts(_) => {
            specialize_index_u64::<H, index_bench::workload::Ts, V>(config)
        }
        index_bench::workload::Key::Ipv4 => {
            specialize_index_u64::<H, index_bench::workload::Ipv4, V>(config)
        }
        index_bench::workload::Key::Snowflake => {
            specialize_index_u64::<H, index_bench::workload::Snowflake, V>(config)
        }
        index_bench::workload::Key::Email => {
            // specialize_index_str::<H, index_bench::workload::Email>(config)
            specialize_index_str::<H, index_bench::workload::Email, V>(config)
        }
        index_bench::workload::Key::Url => {
            // specialize_index_str::<H, index_bench::workload::Url>(config)
            specialize_index_string::<H, index_bench::workload::Url, V>(config)
        }
    }
}

fn specialize_index_u64<
    H: index_bench::index::Hasher,
    K: index_bench::workload::KeyDistribution<Key = u64>,
    V: index_bench::index::Value + ::arctic::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index_bench::index::Name::Art => {
            index_bench::run::<K, u64, art_sys::Rowex<K::Key>, H>(config)
        }
        index_bench::index::Name::Arctic => {
            index_bench::run::<K, V, index_bench::index::ArcticMap<K::Key, V>, H>(config)
        }
        index_bench::index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<K::Key, u64, H>, H>(config)
        }
        index_bench::index::Name::FbTree => {
            index_bench::run::<K, u64, fbtree_sys::FbU64, H>(config)
        }
        index_bench::index::Name::Wormhole => {
            index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config)
        }
    }
}

#[allow(unused)]
fn specialize_index_str<
    H: index_bench::index::Hasher,
    K: index_bench::workload::KeyDistribution<Key = Vec<u8>>,
    V: index_bench::index::Value + ::arctic::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index_bench::index::Name::Art => {
            index_bench::run::<K, u64, art_sys::Rowex<Vec<u8>>, H>(config)
        }
        index_bench::index::Name::Arctic => {
            index_bench::run::<K, V, index_bench::index::ArcticMap<Vec<u8>, V>, H>(config)
        }
        index_bench::index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<&'static [u8], u64, H>, H>(config)
        }
        index_bench::index::Name::FbTree => {
            index_bench::run::<K, u64, fbtree_sys::FbString, H>(config)
        }
        index_bench::index::Name::Wormhole => {
            index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config)
        }
    }
}

#[allow(unused)]
fn specialize_index_string<
    H: index_bench::index::Hasher,
    K: index_bench::workload::KeyDistribution<Key = Vec<u8>>,
    V: index_bench::index::Value + ::arctic::Value + Send + Sync,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index_bench::index::Name::Art => {
            index_bench::run::<K, u64, art_sys::Rowex<Vec<u8>>, H>(config)
        }
        index_bench::index::Name::Arctic => {
            index_bench::run::<K, V, index_bench::index::ArcticMap<Vec<u8>, V>, H>(config)
        }
        index_bench::index::Name::DashMap => {
            index_bench::run::<K, u64, dashmap::DashMap<Vec<u8>, u64, H>, H>(config)
        }
        index_bench::index::Name::FbTree => {
            index_bench::run::<K, u64, fbtree_sys::FbString, H>(config)
        }
        index_bench::index::Name::Wormhole => {
            index_bench::run::<K, u64, wormhole_sys::Wormhole, H>(config)
        }
    }
}
