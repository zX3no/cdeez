#[derive(Debug)]
pub struct Location<'a> {
    pub path: &'a str,
    pub count: usize,
}

pub const fn to_ascii_lowercase(c: u8) -> u8 {
    const ASCII_CASE_MASK: u8 = 0b0010_0000;
    c | (matches!(c, b'A'..=b'Z') as u8 * ASCII_CASE_MASK)
}

#[inline(always)]
pub fn as_str(bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}

pub const MAX_PATH: usize = 260;
// const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;

// #[link(name = "user32")]
#[link(name = "shlwapi")]
extern "system" {
    // fn GetFileAttributesA(lpFileName: *const i8) -> u32;
    pub fn PathFileExistsA(lpFileName: *const i8) -> i32;
}

pub const LOCATIONS: [Location; 40] = [
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\cdeez",
        count: 22,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects",
        count: 246,
    },
    Location {
        path: "\\\\?\\C:\\Users\\Bay\\.cargo",
        count: 1,
    },
    Location {
        path: "\\\\?\\C:\\Users\\Bay\\.cargo\\build_cache",
        count: 1,
    },
    Location {
        path: "\\\\?\\C:\\Users\\Bay\\.cargo\\build_cache\\release",
        count: 7,
    },
    Location {
        path: "\\\\?\\D:\\Desktop",
        count: 4,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\draw",
        count: 3,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\draw\\dx11",
        count: 1,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\lite",
        count: 3,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\lite\\src",
        count: 1,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\lite\\data",
        count: 1,
    },
    Location {
        path: "\\\\?\\D:\\OneDrive",
        count: 1,
    },
    Location {
        path: "\\\\?\\Z:\\Config\\nvim",
        count: 2,
    },
    Location {
        path: "\\\\?\\Z:\\Config",
        count: 3,
    },
    Location {
        path: "\\\\?\\D:\\Ableton",
        count: 6,
    },
    Location {
        path: "\\\\?\\Z:\\",
        count: 3,
    },
    Location {
        path: "\\\\?\\D:\\",
        count: 113,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\god",
        count: 3,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\text-rendering-stuff",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\odin",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\win32",
        count: 2,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\win32\\src",
        count: 1,
    },
    Location {
        path: "\\\\?\\C:\\",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\rss\\src",
        count: 1,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\rss",
        count: 2,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\projects\\win32",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\sfnt",
        count: 3,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\git",
        count: 5,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\cdeez\\projects",
        count: 9,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\cdeez\\god",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\cdeez\\stfnt",
        count: 4,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\cdeez\\sfnt",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\stfnt",
        count: 4,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\stfnt",
        count: 2,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\",
        count: 3,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\god",
        count: 1,
    },
    Location {
        path: "\\\\?\\z:\\config",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\ableton",
        count: 1,
    },
    Location {
        path: "\\\\?\\d:\\desktop\\projects\\cdeez",
        count: 1,
    },
    Location {
        path: "\\\\?\\D:\\Desktop\\projects\\cdeez\\dead",
        count: 1,
    },
];

pub fn fast() {
    unsafe {
        use core::hint::black_box;

        //Get the user input (args).
        //Get current directory (pwd).

        //Replace '/' with '\\'
        //cdeez\\src

        //Combine pwd wih args
        //D:\\Desktop\\projects\\ + cdeez\\src

        //Search database for cdeez
        //Append cdeez\\src to string
        //Check if the path exists.

        // const PWD: &str = r"D:\\Desktop\\projects";
        // const ARGS: &str = "cdeez/src/test";

        //Putting the args at the end could be useful.
        //[...........cdeez/src]
        //[pwd........cdeez/src]
        //[location......../src]

        // let pwd = black_box(PWD);
        let pwd_path = std::env::current_dir().unwrap();
        let pwd = pwd_path.as_os_str().to_str().unwrap();

        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.is_empty() {
            return println!("Argument required.");
        }
        let args = args[0].as_str();
        let locations = black_box(LOCATIONS);

        let pwd_len = pwd.len();
        let args_len = args.len();

        assert!(pwd_len + args_len <= MAX_PATH);

        let mut slash_index = None;
        let mut args_slice: [u8; MAX_PATH + 1] = [0; MAX_PATH + 1];

        let args_start = MAX_PATH - args_len;
        for (i, char) in args.chars().enumerate() {
            let i = args_start + i;
            match char {
                '\\' if slash_index.is_none() => {
                    slash_index = Some(i);
                    args_slice[i] = '\\' as u8;
                }
                '/' => {
                    if slash_index.is_none() {
                        slash_index = Some(i);
                    }
                    args_slice[i] = '\\' as u8;
                }
                _ => args_slice[i] = to_ascii_lowercase(char as u8),
            }
        }

        args_slice[args_start - 1] = b'\\';
        let end = args_start - 1;
        let start = end - pwd.len();
        let mut i = 0;
        let pwd_bytes = pwd.as_bytes();
        for j in start..end {
            args_slice[j] = pwd_bytes[i];
            i += 1;
        }

        let path = as_str(&args_slice[start..]);
        if PathFileExistsA(path.as_ptr() as *const i8) == 1 {
            println!("Found path: {}", path);
            return;
        }

        let index = slash_index.unwrap_or(args_start + args_len);
        let first = &args_slice[args_start..index];

        for location in locations {
            if location.path.as_bytes().ends_with(first) {
                let path_len = location.path.len();
                let start = index - path_len;

                //Currently this copy overrides 'projects' in 'projects/cdeez'
                args_slice[start..index].copy_from_slice(location.path.as_bytes());

                let path = as_str(&args_slice[start..]);
                if PathFileExistsA(path.as_ptr() as *const i8) == 1 {
                    println!("Found path: {}", path);
                    return;
                }

                //TODO: Continute;
                return;
            }
        }

        println!("cdeez: could not find '{}'", &args);
    }
}
