use std::io::BufReader;
use std::io::BufWriter;

fn main() -> anyhow::Result<()> {
    let mut stdin = BufReader::new(std::io::stdin().lock());

    let config: index_bench::Config = serde_json::from_reader(&mut stdin)?;
    let measurement = match config.index {
        index_bench::index::Config::Art => index_bench::run::<index_bench::index::Art>(config)?,
        index_bench::index::Config::Scc => {
            index_bench::run::<index_bench::index::scc::Map>(config)?
        }
    };

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut stdout, &measurement)?;
    Ok(())
}
