use std::io::BufReader;
use std::io::BufWriter;

fn main() -> anyhow::Result<()> {
    let mut stdin = BufReader::new(std::io::stdin().lock());
    let config: index_bench::Config = serde_json::from_reader(&mut stdin)?;
    let measurement = specialize_key(config)?;
    let mut stdout = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut stdout, &measurement)?;
    Ok(())
}

fn specialize_key(config: index_bench::Config) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.key {
        index_bench::workload::Key::U64 => specialize_hash::<index_bench::workload::U64>(config),
        index_bench::workload::Key::Email => {
            specialize_hash::<index_bench::workload::Email>(config)
        }
        index_bench::workload::Key::Prefix(_) => {
            specialize_hash::<index_bench::workload::Prefix>(config)
        }
    }
}

fn specialize_hash<K: index_bench::workload::KeyDistribution>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.hash {
        index_bench::index::Hash::RapidHash => {
            specialize_index::<K, rapidhash::fast::RandomState>(config)
        }
    }
}

fn specialize_index<K: index_bench::workload::KeyDistribution, H: index_bench::index::Hasher>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index_bench::index::Name::Arctic => {
            index_bench::run::<K, index_bench::index::Arctic<K::Key>, H>(config)
        }
        index_bench::index::Name::Bonsai => {
            index_bench::run::<K, index_bench::index::kaist::Bonsai<K::Key>, H>(config)
        }
        index_bench::index::Name::BzTree => {
            index_bench::run::<K, index_bench::index::bz_tree::Map<K::Key>, H>(config)
        }
        index_bench::index::Name::ConcurrentMap => {
            index_bench::run::<K, index_bench::index::concurrent_map::Map<K::Key>, H>(config)
        }
        index_bench::index::Name::Congee => {
            index_bench::run::<K, index_bench::index::congee::Map<K::Key>, H>(config)
        }
        index_bench::index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, index_bench::index::crossbeam_skiplist::Map<K::Key>, H>(config)
        }
        index_bench::index::Name::Papaya => {
            index_bench::run::<K, index_bench::index::papaya::Map<K::Key, H>, H>(config)
        }
        index_bench::index::Name::Scc => {
            index_bench::run::<K, index_bench::index::scc::Map<K::Key, H>, H>(config)
        }
    }
}
