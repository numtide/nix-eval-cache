use blake2::{Blake2s256, Digest};
use nix::fcntl::{fcntl, FcntlArg, FdFlag};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use std::{env, fs};
use tempfile::{NamedTempFile, TempDir};

const LIBREDIRECT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/librecordaccess.so"));

fn cache_dir() -> PathBuf {
    // TODO: macos
    if let Some(cache_dir) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(cache_dir).join("nix-eval-cache");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join("nix-eval-cache");
    }

    // In CI?
    return PathBuf::from(".nix-eval-cache");
}

fn get_cache_key(args: &[String]) -> Result<String, std::io::Error> {
    let mut hasher = Blake2s256::new();
    for arg in &args[1..] {
        hasher.update(arg);
    }
    hasher.update(env::current_dir()?.as_os_str().as_bytes());
    let res = hasher.finalize();
    Ok(hex::encode(res))
}

fn check_cache(cache_key: &Path) -> Result<bool, std::io::Error> {
    let cache_metadata = match fs::metadata(cache_key) {
        Ok(c) => c,
        Err(_) => return Ok(false)
    };
    let cache_time = cache_metadata.modified().unwrap();
    if let Ok(file) = File::open(cache_key) {
        let buf_read = BufReader::new(file);
        let mut found_seperator = false;
        for line in buf_read.split(b'\0') {
            let line = line?;
            if line.is_empty() {
                found_seperator = true;
                continue;
            } else if found_seperator {
                let p = PathBuf::from(OsStr::from_bytes(&line));
                return Ok(p.exists());
            } else {
                let path = PathBuf::from(OsStr::from_bytes(&line));
                match path.metadata() {
                    Ok(s) => {
                        let timestamp = s.modified().unwrap();
                        // file is newer than cache
                        if timestamp > cache_time {
                            return Ok(false)
                        }
                    },
                    _ => { // file was removed, recalculate
                        return Ok(false)
                    }
                };
            }
        }
    }
    return Ok(false);
}

// TODO: better error handling and nicer error messages...
fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();
    let tmp_dir = TempDir::new()?;
    let library = tmp_dir.path().join("librecordaccess.so");
    std::fs::write(&library, LIBREDIRECT)?;

    if args.len() < 2 {
        eprintln!("USAGE: {} resultfile NIX_COMMAND...", args[0]);
        exit(1);
    }
    let resultfile = &args[1];
    let cache_key = get_cache_key(&args[2..])?;
    let dir = cache_dir();
    let cache_file = dir.join(cache_key);

    if PathBuf::from(&resultfile).exists() && check_cache(&cache_file)? {
        println!("skip build");
        return Ok(());
    }

    fs::create_dir_all(&dir)?;

    let write_file = NamedTempFile::new_in(&dir)?;

    fcntl(write_file.as_raw_fd(), FcntlArg::F_SETFD(FdFlag::empty()))?;

    let status = Command::new(&args[2])
        .args(&args[3..])
        .env("LD_PRELOAD", library)
        .env("NIX_EVAL_CACHE_FD", write_file.as_raw_fd().to_string())
        .status()?;
    let store_path = fs::canonicalize(resultfile)?;

    write_file.as_file().write_all(b"\0")?;
    write_file
        .as_file()
        .write_all(store_path.as_os_str().as_bytes())?;
    write_file.persist(cache_file)?;

    match status.code() {
        Some(code) => exit(code),
        // FIXME, also terminate ourselves by a signal
        None => println!("Process terminated by signal"),
    }

    Ok(())
}
