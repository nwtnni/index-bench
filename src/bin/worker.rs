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
        index_bench::workload::Key::U64 => specialize_index::<index_bench::workload::U64>(config),
    }
}

fn specialize_index<K: index_bench::workload::KeyDistribution>(
    config: index_bench::Config,
) -> anyhow::Result<index_bench::measure::Global> {
    match config.index {
        index_bench::index::Config::Art => {
            index_bench::run::<K, index_bench::index::Art<K::Key>>(config)
        }
        index_bench::index::Config::Scc => {
            index_bench::run::<K, index_bench::index::scc::Map<K::Key>>(config)
        }
    }
}
