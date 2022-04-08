use nix::fcntl::{self, fcntl, FcntlArg, OFlag, FdFlag};
use nix::unistd;
use std::io::Write;
use std::os::unix::io::AsRawFd;
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

// TODO: better error handling and nicer error messages...
fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();
    let tmp_dir = TempDir::new()?;
    let library = tmp_dir.path().join("librecordaccess.so");
    std::fs::write(&library, LIBREDIRECT)?;

    if args.len() < 2 {
        eprintln!("USAGE: {} NIX_COMMAND...", args[0]);
        exit(1);
    }

    let dir = cache_dir();
    fs::create_dir_all(&dir)?;

    let mut write_file = NamedTempFile::new_in(&dir)?;

    fcntl(
        write_file.as_raw_fd(),
        FcntlArg::F_SETFD(FdFlag::empty()),
    )?;
    write_file.write_all(b"foo")?;
    //println!("pause!");
    //unistd::pause();

    let status = Command::new(&args[1])
        .args(&args[2..])
        .env("LD_PRELOAD", library)
        .env("NIX_EVAL_CACHE_FD", write_file.as_raw_fd().to_string())
        .status()?;
    // TODO, hash this in future
    let path = format!("{}", env::current_dir()?.display())[1..].replace("/", "-");
    write_file.persist(dir.join(path))?;

    match status.code() {
        Some(code) => exit(code),
        // FIXME, also terminate ourselves by a signal
        None => println!("Process terminated by signal"),
    }

    Ok(())
}
