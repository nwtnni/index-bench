use std::env;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write as _;
use std::sync::Arc;

const USAGE: &str = "Usage: email <INPUT> <OUTPUT>";

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
    let valid = regex::bytes::Regex::new(
        r#"[a-zA-Z0-9.!#$%&'*+\\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*"#,
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

                    if name.ends_with(".rar") {
                        eprintln!("Skip RAR: {name:?}");
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
                        let email = core::str::from_utf8(r#match.as_bytes()).unwrap();
                        let (username, domain) = email.split_once('@').unwrap();

                        for segment in domain.split('.').rev() {
                            buffer.push_str(segment);
                            buffer.push('.');
                        }

                        buffer.pop();
                        buffer.push('@');
                        buffer.push_str(username);
                        buffer.push('\n');
                        total += 1;
                        if pin.get_or_insert(&buffer, 0).1 {
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
                }
            }
        })
        .into_iter()
        .count();

    std::thread::sleep(std::time::Duration::from_secs(1));
    let map = Arc::get_mut(&mut map).unwrap();
    let mut iter = map.as_sequential().iter::<arctic::iter::Sorted>();

    while let Some((key, _)) = iter.lend() {
        output.write_all(key.as_bytes()).unwrap();
    }

    Ok(())
}
