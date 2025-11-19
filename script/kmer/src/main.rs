// https://github.com/nicolasgarza/fetchkmer/blob/d910161007cb7e4c3396a49998081db6d6a1f134/src/compress.rs

use std::env;
use std::fs;
use std::io::BufRead as _;
use std::io::BufReader;
use std::io::BufWriter;

use bitstream_io::BigEndian;
use bitstream_io::BitWrite as _;
use bitstream_io::BitWriter;

static USAGE: &str = "Usage: kmer <INPUT> <OUTPUT>";

fn main() -> std::io::Result<()> {
    let input = env::args().nth(1).expect(USAGE);
    let output = env::args().nth(2).expect(USAGE);

    let mut input = fs::File::open(input)
        .map(BufReader::new)
        .expect("Failed to open input");

    let mut output: BitWriter<_, BigEndian> = fs::File::options()
        .write(true)
        .create_new(true)
        .open(output)
        .map(BufWriter::new)
        .map(BitWriter::new)
        .expect("Failed to open output");

    let mut buffer: Vec<u8> = Vec::new();

    // Hard-code for SRR31218470 for now
    loop {
        // Skip header
        if input.skip_until(b'\n')? == 0 {
            break;
        }

        for _ in 0..3 {
            input.read_until(b'\n', &mut buffer)?;
            buffer.pop();
        }

        if buffer.contains(&b'N') {
            eprintln!("Skipping line with N: {:?}", std::str::from_utf8(&buffer));
            buffer.clear();
            continue;
        }

        // Pack bases into 2-bit representation
        for b in buffer.drain(..) {
            output.write::<2, u8>(CONVERT[b as usize])?;
        }
        output.byte_align()?;
    }

    Ok(())
}

static CONVERT: [u8; 256] = {
    let mut table = [u8::MAX; 256];
    table[b'A' as usize] = 0b00;
    table[b'T' as usize] = 0b01;
    table[b'C' as usize] = 0b10;
    table[b'G' as usize] = 0b11;
    table
};
