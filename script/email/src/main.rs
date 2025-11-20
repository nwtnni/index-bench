use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write as _;
use std::sync::Arc;

const USAGE: &str = "Usage: email <INPUT> <OUTPUT>";

static TOTAL: AtomicU64 = AtomicU64::new(0);
static UNIQUE: AtomicU64 = AtomicU64::new(0);

fn main() -> io::Result<()> {
    let input = env::args().nth(1).expect(USAGE);
    let output = env::args().nth(2).expect(USAGE);

    let mut output = File::options()
        .write(true)
        .create_new(true)
        .open(&output)
        .map(BufWriter::new)
        .unwrap();

    // https://stackoverflow.com/a/8829363
    // https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address
    //
    // NOTE: revised to exclude integers in domain
    let valid = regex::bytes::Regex::new(
        r#"[a-zA-Z0-9.!#$%&'*+\\/=?^_`{|}~-]+@[a-zA-Z](?:[a-zA-Z]{0,61}[a-zA-Z])?(?:\.[a-zA-Z](?:[a-zA-Z]{0,61}[a-zA-Z])?)*"#,
    ).unwrap();

    let mut map = Arc::new(arctic::concurrent::Map::<String, u64>::new());

    jwalk::WalkDir::new(input)
        .process_read_dir({
            let map = Arc::clone(&map);
            move |_depth, path, _state, children| {
                for child in children {
                    let child = child.as_mut().unwrap();
                    let Some(name) = child.file_name().to_str() else {
                        eprintln!("Skip non-UTF8 path: {:?}", child.file_name());
                        continue;
                    };

                    if name.ends_with(".rar") || name.ends_with(".ZIP") {
                        eprintln!("Skip archive: {name:?}");
                        continue;
                    }

                    if !child.file_type().is_file() {
                        continue;
                    }

                    let path = path.join(child.file_name());
                    let data = std::fs::read(&path).unwrap();
                    let mut pin = map.pin();
                    let mut buffer = String::new();
                    let mut unique = 0;
                    let mut total = 0;

                    for r#match in valid.find_iter(&data) {
                        // https://stackoverflow.com/a/574698
                        // https://www.rfc-editor.org/errata_search.php?rfc=3696
                        if r#match.as_bytes().len() > 254 {
                            continue;
                        }

                        let email = core::str::from_utf8(r#match.as_bytes())
                            .expect("Email regex is valid UTF-8");

                        let (username, domain) =
                            email.split_once('@').expect("Email regex has one @");

                        // https://www.rfc-editor.org/errata_search.php?rfc=3696
                        if username.len() > 64 || domain.len() > 255 {
                            continue;
                        }

                        for segment in domain.split('.').rev() {
                            buffer.push_str(segment);
                            buffer.push('.');
                        }

                        buffer.pop();
                        buffer.push('@');
                        buffer.push_str(username);
                        buffer.push('\n');
                        buffer.make_ascii_lowercase();

                        total += 1;
                        if !pin.get_or_insert(&buffer, 0).1 {
                            unique += 1;
                        }

                        buffer.clear();
                    }

                    eprintln!(
                        "{:?}: {}/{} ({:.02}%)",
                        path,
                        unique,
                        total,
                        (unique * 100) as f32 / total as f32
                    );

                    UNIQUE.fetch_add(unique, Ordering::Relaxed);
                    TOTAL.fetch_add(total, Ordering::Relaxed);
                }
            }
        })
        .into_iter()
        .count();

    let map = loop {
        // Wait for `Arc` to be dropped
        match Arc::get_mut(&mut map) {
            Some(map) => break map,
            None => std::thread::sleep(std::time::Duration::from_secs(1)),
        }
    };

    let unique = UNIQUE.load(Ordering::Relaxed);
    let total = TOTAL.load(Ordering::Relaxed);

    eprintln!(
        "{}/{} ({:.02}%)",
        unique,
        total,
        (unique * 100) as f32 / total as f32
    );

    let mut iter = map.as_sequential().iter::<arctic::iter::Sorted>();
    while let Some((key, _)) = iter.lend() {
        output.write_all(key.as_bytes()).unwrap();
    }

    Ok(())
}
