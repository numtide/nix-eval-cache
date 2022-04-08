use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::{read_dir, DirEntry};
use std::io;

#[macro_export]
macro_rules! ok(($expression:expr) => ($expression.unwrap()));

#[macro_export]
macro_rules! log {
    ($fmt:expr) => (println!(concat!("build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("build.rs:{}: ", $fmt),
    line!(), $($arg)*));
}

pub fn run<F>(name: &str, mut configure: F)
where
    F: FnMut(&mut Command) -> &mut Command,
{
    let mut command = Command::new(name);
    let configured = configure(&mut command);
    log!("Executing {:?}", configured);
    if !ok!(configured.status()).success() {
        panic!("failed to execute {:?}", configured);
    }
    log!("Command {:?} finished successfully", configured);
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

pub fn rebuild_if_dir_changed(dir: &Path) {
    visit_dirs(dir, &|e| {
        println!("cargo:rerun-if-changed={}", e.path().display())
    })
    .expect("failed to list dir");
}

fn main() {
    let curdir = env::current_dir().expect("cannot get current working directory");
    let libdir = curdir.join("recordaccess");

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR is not set");
    run("cargo", |command| {
        command
            .arg("build")
            .arg("--release")
            .current_dir(&libdir)
    });
    let dynamiclib = Path::new(&out_dir).join("librecordaccess.so");

    let cc = env::var("CC").unwrap_or("cc".to_string());
    // FIXME, this library is bigger than it needs to be... but we need to
    // somehow link rust and c into one shared library
    run(&cc, |command| {
        command
            .arg("-shared")
            .arg("-O2")
            .arg("-pthread")
            .arg("-o")
            .arg(&dynamiclib)
            .arg(curdir.join("recordaccess/src/redirectopen.c"))
            .arg("-lrecordaccess")
            .arg("-ldl")
            .arg(&format!("-L{}", curdir.join("recordaccess/target/release").display()))
    });

    println!("cargo:rerun-if-changed=recordaccess/build.rs");
    println!("cargo:rerun-if-changed=recordaccess/Cargo.toml");
    rebuild_if_dir_changed(&curdir.join("recordaccess/src"));
}
