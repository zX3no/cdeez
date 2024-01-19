#![feature(file_create_new)]
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::from_utf8_unchecked,
};

#[derive(Debug)]
struct Location<'a> {
    path: &'a str,
    count: usize,
}

fn read_db(db: &str) -> Vec<Location> {
    db.lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let end = line.find("\" ").unwrap();
            let path = unsafe { from_utf8_unchecked(&line.as_bytes()[1..end]) };
            let count = unsafe { from_utf8_unchecked(&line.as_bytes()[end + 2..]) }
                .parse::<usize>()
                .unwrap();
            Location { path, count }
        })
        .collect()
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

        writeln!(&mut writer, r#""{}" {}"#, location.path, location.count).unwrap()
    }

    if !found {
        writeln!(&mut writer, r#""{}" 1"#, path.display()).unwrap();
    }
}

#[cfg(target_os = "windows")]
fn main() {
    let db_path = Path::new(&std::env::var("APPDATA").unwrap()).join(Path::new("cdeez\\cdeez.db"));

    //Make sure the directory and database exists.
    let _ = std::fs::create_dir(db_path.parent().unwrap());
    let _ = File::create_new(db_path.as_path());

    let db = std::fs::read_to_string(&db_path).unwrap();
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return;
    }

    if let "--list" = args[0].as_str() {
        println!("cdeez: --list");
        for line in db.lines() {
            println!("{line}");
        }
        return;
    }

    let pwd = std::env::current_dir().unwrap();
    let new = pwd.join(&args[0]);
    let mut locations = read_db(&db);

    let path = match std::fs::canonicalize(&new) {
        //Files cannot contain ':', the user must want a drive.
        _ if args[0].ends_with(':') && args[0].len() == 2 => {
            match std::fs::canonicalize(format!("{}\\", &args[0])) {
                Ok(path) => path,
                Err(_) => return println!("cdeez: cannot cd file '{}'", new.display()),
            }
        }
        Ok(path) if path.is_file() => {
            println!("cdeez: cannot cd file '{}'", new.display());
            locations.retain(|loc| Path::new(&loc.path) != path);
            return write_config(&path, &db_path, locations);
        }
        //User wants to navigate to a directory in the current folder.
        Ok(path) => path,
        //User wants to go somewhere else.
        Err(_) => {
            let mut path = None;
            let mut remove = false;

            //TODO: Linux paths are case sensitive. 🙄
            let user_input = &args[0].to_ascii_lowercase();
            let normalized = user_input.replace('\\', "/");

            let splits = if normalized.contains('/') {
                Some(normalized.split('/').collect::<Vec<&str>>())
            } else {
                None
            };

            'a: for l in locations.iter_mut() {
                let lower = l.path.to_ascii_lowercase();
                let target = PathBuf::from(&lower);

                if target.ends_with(&normalized) && splits.is_none() {
                    if !target.exists() {
                        remove = true;
                    }
                    path = Some(target);
                    break;
                }

                let Some(splits) = &splits else {
                    continue;
                };

                //Handle cases where 'foo' exists in the database but 'foo/bar' does not.
                let mut p = target;
                for split in splits {
                    p = p.join(split);
                    if !p.exists() {
                        continue 'a;
                    }
                }

                path = Some(p);
                break;
            }

            let path = if let Some(path) = path {
                path
            } else {
                //If the user entered a letter a..z they most likely wanted a drive.
                let lowercase = args[0].as_bytes()[0].to_ascii_lowercase();
                if args[0].len() == 1 && matches!(lowercase, b'a'..=b'z') {
                    let path = format!("{}:\\", lowercase as char);
                    match std::fs::canonicalize(&path) {
                        Ok(path) => path,
                        Err(_) => return println!("cdeez: cannot cd file '{}'", path),
                    }
                } else {
                    return println!("cdeez: cannot cd file '{}'", new.display());
                }
            };

            //Path exists in database but not on file system.
            if remove {
                println!("cdeez: removing dead path '{}'", &args[0]);
                locations.retain(|loc| Path::new(&loc.path.to_ascii_lowercase()) != path);

                //Update the config removing the dead path.
                let file = File::create(db_path).expect("Unable to create file");
                let mut writer = BufWriter::new(file);
                for location in locations {
                    writeln!(&mut writer, r#""{}" {}"#, location.path, location.count).unwrap();
                }
                return;
            }

            path
        }
    };

    write_config(&path, &db_path, locations);
    println!("{}", path.display());
}
