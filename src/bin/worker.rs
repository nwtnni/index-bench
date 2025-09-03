use std::io::BufReader;
use std::io::BufWriter;

fn main() -> anyhow::Result<()> {
    let mut stdin = BufReader::new(std::io::stdin().lock());

    let config = serde_json::from_reader(&mut stdin)?;
    let measurement = index_bench::run(config)?;

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut stdout, &measurement)?;
    Ok(())
}
