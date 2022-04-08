use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::unistd::pipe;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::Command;
use tempfile::TempDir;
use std::str;
use std::process::exit;

const LIBREDIRECT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/librecordaccess.so"));

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

    let (read_file, write_file) = {
        let (read_fd, write_fd) = pipe()?;
        unsafe { (File::from_raw_fd(read_fd), File::from_raw_fd(write_fd)) }
    };
    fcntl(read_file.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_CLOEXEC))?;

    let mut child = Command::new(&args[1])
        .args(&args[2..])
        .env("LD_PRELOAD", library)
        .env("NIX_EVAL_CACHE_FD", write_file.as_raw_fd().to_string())
        .spawn()?;

    drop(write_file);
    let reader = BufReader::new(read_file);

    for path in reader.split(b'\0') {
        if let Ok(path) = str::from_utf8(&path?) {
            println!("{}", path);
        }
    }
    let status = child.wait()?;
    match status.code() {
        Some(code) => exit(code),
        // FIXME, also terminate ourselves by a signal
        None       => println!("Process terminated by signal")
    }

    Ok(())
}
