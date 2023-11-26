use cdeez::*;
use divan::black_box;
use std::path::PathBuf;

fn main() {
    divan::main();
}

#[divan::bench]
fn old() {
    const ARGS: &str = "cdeez/src";
    let pwd = std::env::current_dir().unwrap();
    let args = black_box(ARGS);

    let mut locations = black_box(LOCATIONS);

    let new = pwd.join(&args);

    let path = match std::fs::canonicalize(&new) {
        //Files cannot contain ':', the user must want a drive.
        _ if args.ends_with(':') && args.len() == 2 => {
            match std::fs::canonicalize(format!("{}\\", &args)) {
                Ok(path) => path,
                Err(_) => {
                    // println!("cdeez: cannot cd file '{}'", new.display());
                    return;
                }
            }
        }
        Ok(path) if path.is_file() => {
            // println!("cdeez: cannot cd file '{}'", new.display());
            // locations.retain(|loc| Path::new(&loc.path) != path);
            return;
            // return write_config(&path, &db_path, locations);
        }
        //User wants to navigate to a directory in the current folder.
        Ok(path) => path,
        //User wants to go somewhere else.
        Err(_) => {
            let mut path = None;
            let mut remove = false;

            //TODO: Linux paths are case sensitive. 🙄
            let user_input = &args.to_ascii_lowercase();
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

            let Some(path) = path else {
                // println!("cdeez: could not find '{}'", &args);
                return;
            };

            //Path exists in database but not on file system.
            if remove {
                // println!("cdeez: removing dead path '{}'", &args[0]);
                return;
            }

            path
        }
    };
    assert!(path.exists());
}

#[divan::bench]
pub fn fast() {
    cdeez::fast();
}
