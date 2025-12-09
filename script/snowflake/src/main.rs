use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write as _;

fn main() -> anyhow::Result<()> {
    let mut buffer = String::new();

    let out = std::env::args().nth(1).unwrap();
    let mut out = File::options()
        .create_new(true)
        .write(true)
        .open(out)
        .map(BufWriter::new)?;

    for path in std::env::args().skip(2) {
        eprintln!("Processing {}...", path);

        let mut reader = File::open(path).map(BufReader::new)?;

        buffer.clear();
        reader.read_to_string(&mut buffer)?;

        for line in buffer.trim().split('\n').skip(1) {
            let (id, _) = line.split_once(',').unwrap();

            let Ok(id) = id.parse::<u64>() else {
                eprintln!("Invalid ID: {:x?}", id);
                continue;
            };
            out.write_all(&id.to_le_bytes())?;
        }
    }

    Ok(())
}
