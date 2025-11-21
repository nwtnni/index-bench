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
            specialize_key::<rapidhash::fast::RandomState>(config)
        }
    }
}

fn specialize_key<H: index_bench::index::Hasher>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.workload.key {
        index_bench::workload::Key::U64 => {
            specialize_index::<H, index_bench::workload::U64>(config)
        }
        index_bench::workload::Key::Email => {
            todo!()
        }
        index_bench::workload::Key::Url => todo!(),
        // index_bench::workload::Key::Email => {
        //     specialize_hash::<index_bench::workload::Email>(config)
        // }
        // index_bench::workload::Key::Url => specialize_hash::<index_bench::workload::Url>(config),
        // index_bench::workload::Key::Prefix(_) => {
        //     specialize_hash::<index_bench::workload::Prefix>(config)
        // }
        index_bench::workload::Key::Sparse(_) => {
            specialize_index::<H, index_bench::workload::Sparse>(config)
        }
        index_bench::workload::Key::Kmer => {
            specialize_index::<H, index_bench::workload::Kmer>(config)
        }
    }
}

fn specialize_index<
    H: index_bench::index::Hasher,
    K: index_bench::workload::KeyDistribution<Key = u64>,
>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index.name {
        index_bench::index::Name::Art => index_bench::run::<K, art_sys::Rowex<u64>, H>(config),
        index_bench::index::Name::Arctic => {
            index_bench::run::<K, arctic::concurrent::Map<K::Key, u64>, H>(config)
        }
        // index_bench::index::Name::Bonsai => {
        //     index_bench::run::<K, index_bench::index::kaist::BonsaiTreeMap<K::Key, u64>, H>(config)
        // }
        // index_bench::index::Name::BPlusTree => {
        //     index_bench::run::<K, bplustree::BPlusTree<K::Key, u64>, H>(config)
        // }
        // index_bench::index::Name::BzTree => {
        //     index_bench::run::<K, bztree::BzTree<K::Key, u64>, H>(config)
        // }
        index_bench::index::Name::ConcurrentMap => {
            index_bench::run::<K, concurrent_map::ConcurrentMap<K::Key, u64>, H>(config)
        }
        index_bench::index::Name::Congee => {
            index_bench::run::<K, congee::Congee<usize, usize>, H>(config)
        }
        // index_bench::index::Name::Contrie => {
        //     index_bench::run::<K, contrie::CloneConMap<K::Key, u64>, H>(config)
        // }
        index_bench::index::Name::CrossbeamSkiplist => {
            index_bench::run::<K, crossbeam_skiplist::SkipMap<K::Key, u64>, H>(config)
        }
        index_bench::index::Name::DashMap => {
            index_bench::run::<K, dashmap::DashMap<K::Key, u64, H>, H>(config)
        }
        index_bench::index::Name::FbTree => index_bench::run::<K, fbtree_sys::FbU64, H>(config),
        index_bench::index::Name::Papaya => {
            index_bench::run::<K, papaya::HashMap<K::Key, u64, H>, H>(config)
        }
        index_bench::index::Name::SccHashMap => {
            index_bench::run::<K, scc::HashMap<K::Key, u64, H>, H>(config)
        }
        index_bench::index::Name::SccTreeIndex => {
            index_bench::run::<K, scc::TreeIndex<K::Key, u64>, H>(config)
        }
        index_bench::index::Name::Wormhole => {
            index_bench::run::<K, wormhole_sys::Wormhole, H>(config)
        }
    }
}
