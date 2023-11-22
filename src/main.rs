use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug)]
pub struct Location<'a> {
    path: &'a str,
    count: usize,
}

pub fn create_db(db: &str) -> Vec<Location> {
    let lines = db.lines();
    let mut locations = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let end = line.find("\" ").unwrap();
        unsafe {
            let path = std::str::from_utf8_unchecked(&line.as_bytes()[1..end]);
            let count = std::str::from_utf8_unchecked(&line.as_bytes()[end + 2..])
                .parse::<usize>()
                .unwrap();
            locations.push(Location { path, count });
        }
    }
    locations
}

fn main() {
    #[cfg(target_os = "windows")]
    let db_path = Path::new(&std::env::var("APPDATA").unwrap()).join(Path::new("cdeez\\cdeez.db"));

    #[cfg(not(target_os = "windows"))]
    let path = Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join(Path::new("cdeez\\cdeez.db"));

    //Make sure the directory exists.
    let _ = std::fs::create_dir(db_path.parent().unwrap());
    let db = std::fs::read_to_string(&db_path);
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        std::process::exit(1);
    }

    let pwd = std::env::current_dir().unwrap();
    let new = pwd.join(&args[0]);

    let Ok(db) = &db else {
        let Ok(path) = std::fs::canonicalize(&new) else {
            println!("cdeez: could not find {}", new.display());
            std::process::exit(1);
        };

        let file = File::create(db_path.as_path()).unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(b"\"").unwrap();
        writer.write_all(path.to_str().unwrap().as_bytes()).unwrap();
        writer.write_all(b"\" ").unwrap();
        writer.write_all(b"1").unwrap();
        return;
    };

    let mut locations = create_db(&db);

    if locations.is_empty() {
        std::fs::remove_file(db_path).unwrap();
        println!("cdeez: database exists but is empty. this should not happen");
        std::process::exit(1);
    }

    let path = match std::fs::canonicalize(&new) {
        Ok(path) if path.is_file() => {
            println!("cdeez: cannot cd file '{}'", new.display());
            std::process::exit(1);
        }
        //User wants to navigate to a directory in the current folder.
        Ok(path) => path,
        //User wants to go somewhere else.
        Err(_) => {
            let (mut path, mut count) = (None, 0);
            for l in &locations {
                let p = Path::new(l.path);
                if l.count > count && p.ends_with(&args[0]) {
                    path = Some(p);
                    count = l.count;
                }
            }

            let Some(path) = path else {
                println!("cdeez: could not find '{}'", &args[0]);
                std::process::exit(1);
            };

            path.to_path_buf()
        }
    };

    let file = File::create(db_path.as_path()).unwrap();
    let mut writer = BufWriter::new(file);
    let mut found = false;

    for location in locations.iter_mut() {
        if Path::new(&location.path) == path {
            location.count += 1;
            found = true;
        }

        writer.write_all(b"\"").unwrap();
        writer.write_all(location.path.as_bytes()).unwrap();
        writer.write_all(b"\" ").unwrap();
        writer
            .write_all(location.count.to_string().as_bytes())
            .unwrap();
        writer.write_all(b"\n").unwrap();
    }

    if !found {
        writer.write_all(b"\"").unwrap();
        writer.write_all(path.to_str().unwrap().as_bytes()).unwrap();
        writer.write_all(b"\" ").unwrap();
        writer.write_all(b"1").unwrap();
        writer.write_all(b"\n").unwrap();
    }

    //Output the path.
    //TODO: Could this be made faster?
    println!("{}", path.display());
}
