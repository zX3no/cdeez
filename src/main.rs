use std::{
    fs::{self, File},
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

fn main() {
    let args: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");

    if args.is_empty() {
        return;
    }

    #[cfg(target_os = "macos")]
    let (home, db, db_path) = {
        let home = home::home_dir().unwrap();
        let mut db_path = home.clone();
        db_path.push(".config/cdeez");
        db_path.push("cdeez.db");

        let _ = fs::create_dir(db_path.parent().unwrap());
        let _ = fs::File::create_new(db_path.as_path());
        let db = fs::read_to_string(&db_path).unwrap();

        (home, db, db_path)
    };

    #[cfg(target_os = "windows")]
    let (home, db, db_path) = {
        // use std::os::windows::ffi::OsStringExt;
        // pub const FOLDERID_PROFILE: GUID = GUID {
        //     data1: 0x5E6C858F,
        //     data2: 0x0E22,
        //     data3: 0x4760,
        //     data4: [0x9A, 0xFE, 0xEA, 0x33, 0x17, 0xB6, 0x71, 0x73],
        // };

        // #[repr(C)]
        // pub struct GUID {
        //     pub data1: u32,
        //     pub data2: u16,
        //     pub data3: u16,
        //     pub data4: [u8; 8],
        // }

        // #[link(name = "shell32")]
        // extern "system" {
        //     pub fn SHGetKnownFolderPath(
        //         rfid: *const GUID,
        //         dwFlags: u32,
        //         hToken: *mut std::ffi::c_void,
        //         pszPath: *mut *mut u16,
        //     ) -> i32;
        // }

        // let home = unsafe {
        //     let mut path: *mut u16 = std::ptr::null_mut();
        //     let result =
        //         SHGetKnownFolderPath(&FOLDERID_PROFILE, 0, std::ptr::null_mut(), &mut path);
        //     assert!(result == 0);
        //     let slice = std::slice::from_raw_parts(path, {
        //         let mut len = 0;
        //         while *path.offset(len) != 0 {
        //             len += 1;
        //         }
        //         len as usize
        //     });
        //     std::ffi::OsString::from_wide(slice)
        // };

        let home = std::env::var("APPDATA").unwrap();
        let db_path = Path::new(&home).join(Path::new("cdeez\\cdeez.db"));

        //Make sure the directory and database exists.
        let _ = fs::create_dir(db_path.parent().unwrap());
        let _ = fs::File::create_new(db_path.as_path());
        let db = fs::read_to_string(&db_path).unwrap();

        (PathBuf::from(home), db, db_path)
    };

    let td = args.replace("~", home.to_str().unwrap());

    if let "--list" = td.as_str() {
        println!("cdeez: --list");
        for line in db.lines() {
            println!("{line}");
        }
        return;
    }

    //Check if the path is absolute (~/, /Users/) before adding the current directory to it.
    //user type cdeez, pwd is /Users/Desktop, the user wants /Users/Desktop/cdeez.
    //There might be issues with this.
    let target_path = match Path::new(&td).exists() {
        true => PathBuf::from(&td),
        false => std::env::current_dir().unwrap().join(&td),
    };

    let mut locations = read_db(&db);

    let path = match fs::canonicalize(&target_path) {
        //Files cannot contain ':', the user must want a drive.
        //I don't know how this shit works on apples.
        #[cfg(target_os = "windows")]
        _ if td.ends_with(':') && td.len() == 2 => {
            let drive = format!("{}\\", &td);
            match fs::canonicalize(&drive) {
                Ok(path) => path,
                Err(_) => return println!("cdeez: cannot cd drive '{}'", drive),
            }
        }
        Ok(path) if path.is_file() => {
            println!("cdeez: cannot cd to file '{}'", target_path.display());
            locations.retain(|loc| Path::new(&loc.path) != path);
            return write_config(&path, &db_path, locations);
        }
        //User wants to navigate to a directory in the current folder.
        Ok(path) => path,
        //User wants to go somewhere else.
        Err(_) => {
            let mut path = None;
            let mut remove = false;

            // #[cfg(target_os = "linux")]
            // let user_input = &td;

            #[cfg(target_os = "macos")]
            let user_input = &td.to_ascii_lowercase();

            #[cfg(target_os = "windows")]
            let user_input = &td.to_ascii_lowercase().replace("\\", "/");

            let splits: Option<Vec<&str>> = user_input
                .contains('/')
                .then(|| user_input.split('/').collect());

            'l: for l in locations.iter_mut() {
                let lower = l.path.to_ascii_lowercase();
                let target = PathBuf::from(&lower);

                if target.ends_with(&user_input) && splits.is_none() {
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
                        continue 'l;
                    }
                }

                path = Some(p);
                break;
            }

            #[cfg(target_os = "macos")]
            let path = match path {
                Some(path) => path,
                None => return println!("cdeez: cannot find folder '{}'", target_path.display()),
            };

            #[cfg(target_os = "windows")]
            let path = if let Some(path) = path {
                path
            } else {
                //If the user entered a letter a..z they most likely wanted a drive.
                let lowercase = td.as_bytes()[0].to_ascii_lowercase();
                if td.len() == 1 && matches!(lowercase, b'a'..=b'z') {
                    let path = format!("{}:\\", lowercase as char);
                    match fs::canonicalize(&path) {
                        Ok(path) => path,
                        Err(_) => return println!("cdeez: cannot cd drive '{}'", path),
                    }
                } else {
                    return println!("cdeez: cannot find folder '{}'", target_path.display());
                }
            };

            //Path exists in database but not on file system.
            if remove {
                println!("cdeez: removing dead path '{}'", &td);
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

    println!("{}", path.display());
    write_config(&path, &db_path, locations);
}
