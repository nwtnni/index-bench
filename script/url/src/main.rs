use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write as _;

use bytes::Buf as _;
use libflate::gzip;

fn main() -> std::io::Result<()> {
    let path = env::args().nth(1).expect("USAGE: url <OUTPUT>");

    let mut out = File::create_new(path)
        .map(BufWriter::new)
        .map(gzip::Encoder::new)
        .unwrap()
        .unwrap();

    let mut buffer = Vec::new();
    let mut len = hdrhistogram::Histogram::<u32>::new(3).unwrap();
    let client = reqwest::blocking::Client::builder()
        .user_agent("nwtnni (nwtnni@gmail.com)")
        .build()
        .unwrap();
    let mut url = String::from(
        "https://data.commoncrawl.org/cc-index/collections/CC-MAIN-2026-12/indexes/cdx-",
    );

    const CHUNK_LEN: usize = 3_000;
    const URLS: usize = 100_000_000usize.next_multiple_of(CHUNK_LEN);
    const CHUNK_COUNT: usize = URLS / CHUNK_LEN;

    for chunk in 0..URLS / CHUNK_LEN {
        write!(&mut url, "{chunk:05}.gz").unwrap();

        let response = client.get(&url).send().unwrap().error_for_status().unwrap();
        let body = response.bytes().unwrap();

        println!(
            "{} / {} | {:.02}% | {} MiB",
            chunk,
            CHUNK_COUNT,
            chunk as f64 / CHUNK_COUNT as f64,
            body.len() / 1024 / 1024,
        );

        let mut reader = gzip::Decoder::new(body.reader())
            .map(BufReader::new)
            .unwrap();

        for _ in 0..CHUNK_LEN {
            reader.read_until(b' ', &mut buffer).unwrap();
            reader.skip_until(b'\n').unwrap();

            buffer.pop();
            len.record(buffer.len() as u64).unwrap();

            buffer.push(b'\n');
            out.write_all(&buffer).unwrap();

            buffer.clear();
        }

        url.truncate(url.len() - 8);
    }

    // let total = len.len();
    // println!(
    //     "valid = {}/{} ({:.02}%)",
    //     len.len(),
    //     total,
    //     (len.len() * 100) as f32 / total as f32
    // );

    println!("count = {}", len.len());
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
