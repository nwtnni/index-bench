use core::hash::Hasher as _;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const COUNT: usize = 100_000_000;

fn main() {
    let version = env::args().nth(1).expect("Expected version (4 or 7)");
    let mut out = File::create_new("uuid.bin").map(BufWriter::new).unwrap();
    let mut hasher = rapidhash::fast::RapidHasher::default();

    match version.as_str() {
        "4" => {
            hasher.write_usize(4);

            for i in 0..COUNT {
                hasher.write_usize(i);
                let lo = hasher.finish();
                hasher.write_usize(i);
                let hi = hasher.finish();

                let uuid = uuid::Builder::from_random_bytes(
                    ((lo as u128) | ((hi as u128) << 64)).to_ne_bytes(),
                )
                .with_version(uuid::Version::Random)
                .with_variant(uuid::Variant::RFC4122)
                .into_uuid()
                .as_u128();

                out.write_all(&uuid.to_le_bytes()).unwrap();
            }
        }
        "7" => {
            hasher.write_usize(7);

            for i in 0..COUNT {
                hasher.write_usize(i);
                let lo = hasher.finish();
                hasher.write_usize(i);
                let hi = hasher.finish();
                let random = ((lo as u128) | ((hi as u128) << 64)).to_ne_bytes();

                let now: u64 = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .try_into()
                    .unwrap();

                let uuid = uuid::Builder::from_unix_timestamp_millis(
                    now,
                    &core::array::from_fn(|i| random[i]),
                )
                .with_version(uuid::Version::SortRand)
                .with_variant(uuid::Variant::RFC4122)
                .into_uuid()
                .as_u128();

                out.write_all(&uuid.to_le_bytes()).unwrap();
            }
        }
        _ => panic!("Unrecognized version {version}"),
    }
}
