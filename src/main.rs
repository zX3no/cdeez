#![feature(extract_if, file_create_new)]
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

#[derive(Debug)]
struct Location<'a> {
    path: &'a str,
    count: usize,
}

fn read_db(db: &str) -> Vec<Location> {
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

fn write_config(path: &Path, db_path: &Path, mut locations: Vec<Location>) {
    let file = File::create(db_path).unwrap();
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
}

fn main() {
    #[cfg(target_os = "windows")]
    let db_path = Path::new(&std::env::var("APPDATA").unwrap()).join(Path::new("cdeez\\cdeez.db"));

    #[cfg(not(target_os = "windows"))]
    let path = Path::new(&std::env::var("HOME").unwrap())
        .join(".config")
        .join(Path::new("cdeez\\cdeez.db"));

    //Make sure the directory and database exists.
    let _ = std::fs::create_dir(db_path.parent().unwrap());
    let _ = File::create_new(db_path.as_path());

    let db = std::fs::read_to_string(&db_path).unwrap();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return;
    }

    match args[0].as_str() {
        "--debug" => {
            println!("cdeez: --debug");
            for line in db.lines() {
                println!("{line}");
            }
            std::process::exit(1);
        }
        _ => {}
    }

    let pwd = std::env::current_dir().unwrap();
    let new = pwd.join(&args[0]);
    let mut locations = read_db(&db);

    let path = match std::fs::canonicalize(&new) {
        Ok(path) if path.is_file() => {
            println!("cdeez: cannot cd file '{}'", new.display());
            let locations: Vec<_> = locations
                .extract_if(|location| Path::new(location.path) != path)
                .collect();
            write_config(&path, &db_path, locations);
            std::process::exit(1);
        }
        //User wants to navigate to a directory in the current folder.
        Ok(path) => path,
        //User wants to go somewhere else.
        Err(_) => {
            let (mut path, mut count) = (None, 0);
            let mut remove = false;

            for l in &locations {
                let p = Path::new(l.path);
                if l.count > count && p.ends_with(&args[0]) {
                    if !p.exists() {
                        remove = true;
                    }

                    path = Some(p);
                    count = l.count;
                }
            }

            let Some(path) = path else {
                println!("cdeez: could not find '{}'", &args[0]);
                std::process::exit(1);
            };

            //Path exists in database but not on file system.
            if remove {
                println!("cdeez: removing dead path '{}'", &args[0]);
                let mut locations: Vec<_> = locations
                    .extract_if(|location| Path::new(location.path) != path)
                    .collect();

                //Update the config removing the dead path.
                let file = File::create(&db_path).unwrap();
                let mut writer = BufWriter::new(file);
                for location in locations.iter_mut() {
                    writer.write_all(b"\"").unwrap();
                    writer.write_all(location.path.as_bytes()).unwrap();
                    writer.write_all(b"\" ").unwrap();
                    writer
                        .write_all(location.count.to_string().as_bytes())
                        .unwrap();
                    writer.write_all(b"\n").unwrap();
                }

                writer.flush().unwrap();
                std::process::exit(1);
            }

            path.to_path_buf()
        }
    };

    write_config(&path, &db_path, locations);

    //Output the path.
    //TODO: Could this be made faster?
    println!("{}", path.display());
}
