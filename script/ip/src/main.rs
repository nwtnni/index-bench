use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write as _;
use std::net::Ipv4Addr;

use bzip2_rs::DecoderReader;

fn main() -> anyhow::Result<()> {
    let mut buffer = String::new();

    let out = std::env::args().nth(1).unwrap();
    let mut out = File::options()
        .create_new(true)
        .write(true)
        .open(out)
        .map(BufWriter::new)?;

    let mut sorted = BTreeSet::new();

    for path in std::env::args().skip(2) {
        let mut reader = File::open(path)
            .map(BufReader::new)
            .map(DecoderReader::new)?;

        reader.read_to_string(&mut buffer)?;

        for line in buffer.split('\n') {
            let Ok(ip) = line.parse::<Ipv4Addr>() else {
                eprintln!("Skipping invalid IP: {line}");
                continue;
            };
            sorted.insert(ip.to_bits());
        }
    }

    for ip in sorted {
        out.write_all(&ip.to_le_bytes())?;
    }

    Ok(())
}
