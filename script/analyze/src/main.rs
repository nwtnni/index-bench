use std::env;
use std::fs::File;
use std::io::BufRead as _;
use std::io::BufReader;

use libflate::gzip;

fn main() -> std::io::Result<()> {
    let path = env::args().nth(1).expect("USAGE: analyze <PATH.gz>");

    let mut file = File::open(path)
        .map(BufReader::new)
        .map(gzip::Decoder::new)
        .expect("Failed to open file")
        // HACK: double-buffer to work around lack of `std::io::BufRead` for `gzip::Decoder`
        .map(BufReader::new)
        .expect("Failed to create gzip decoder");

    let mut buffer = Vec::new();

    let mut invalid = 0;
    let mut len = hdrhistogram::Histogram::<u32>::new(3).unwrap();

    loop {
        buffer.clear();
        if file.read_until(b'\n', &mut buffer)? == 0 {
            break;
        }

        let Ok(string) = core::str::from_utf8(&buffer) else {
            invalid += 1;
            eprintln!(
                "Skipping invalid UTF-8: {:?}",
                String::from_utf8_lossy(&buffer)
            );
            continue;
        };

        len.record(string.len() as u64).unwrap();
    }

    let total = len.len() + invalid;
    println!(
        "valid = {}/{} ({:.02}%)",
        len.len(),
        total,
        (len.len() * 100) as f32 / total as f32
    );

    println!("avg = {}", len.mean());
    println!("std = {}", len.stdev());

    for (name, quantile) in [
        ("min", 0.0),
        ("p50", 0.5),
        ("p75", 0.75),
        ("p90", 0.9),
        ("p99", 0.99),
        ("max", 1.0),
    ] {
        let value = len.value_at_quantile(quantile);
        println!("{name} = {value}");
    }

    Ok(())
}
