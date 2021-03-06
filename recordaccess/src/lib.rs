use lazy_static::lazy_static;
use libc::{c_char, c_int, mode_t};

use std::collections::HashSet;
use std::ffi::{CStr, CString, OsString};
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::sync::Mutex;
use std::{env, fs};

lazy_static! {
    static ref PATHS: Mutex<HashSet<CString>> = Mutex::new(HashSet::new());
    static ref REAL_OPEN: extern "C" fn(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int = unsafe {
        std::mem::transmute(libc::dlsym(
            libc::RTLD_NEXT,
            b"open\0".as_ptr() as *const i8,
        ))
    };
    static ref REAL_OPEN64: extern "C" fn(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int = unsafe {
        std::mem::transmute(libc::dlsym(
            libc::RTLD_NEXT,
            b"open64\0".as_ptr() as *const i8,
        ))
    };
    static ref REAL_OPENAT: extern "C" fn(fd: c_int, path: *const c_char, oflag: c_int, mode: mode_t) -> c_int = unsafe {
        std::mem::transmute(libc::dlsym(
            libc::RTLD_NEXT,
            b"openat\0".as_ptr() as *const i8,
        ))
    };
    static ref REAL_OPENAT64: extern "C" fn(fd: c_int, path: *const c_char, oflag: c_int, mode: mode_t) -> c_int = unsafe {
        std::mem::transmute(libc::dlsym(
            libc::RTLD_NEXT,
            b"openat64\0".as_ptr() as *const i8,
        ))
    };
    static ref NIX_EVAL_CACHE_FILE: Option<Mutex<File>> = {
        if let Ok(strval) = env::var("NIX_EVAL_CACHE_FD") {
            if let Ok(val) = strval.parse::<i32>() {
                Some(unsafe { Mutex::new(File::from_raw_fd(val)) })
            } else {
                None
            }
        } else {
            None
        }
    };
}

pub fn record_path(path: *const c_char) {
    let c_str: &CStr = unsafe { CStr::from_ptr(path) };
    let bytes = &c_str.to_bytes_with_nul();

    let s = unsafe { std::str::from_utf8_unchecked(c_str.to_bytes()) };
    // For debugging
    //println!("{}", s);

    // only consider nix/json files and ignore immutable files in nix store
    if (!bytes.ends_with(b".nix\0") && !bytes.ends_with(b".json\0"))
        || bytes.starts_with(b"/nix/store")
    {
        return;
    }
    match fs::metadata(s) {
        Ok(m) => {
            if !m.file_type().is_file() {
                return;
            }
        }
        Err(e) => {
            return;
        }
    };

    let mut paths = PATHS.lock().unwrap();
    if paths.contains(c_str) {
        return;
    }
    paths.insert(c_str.to_owned());
    if let Some(file) = &*NIX_EVAL_CACHE_FILE {
        let mut file = file.lock().unwrap();
        let _ = file.write_all(bytes);
        let _ = file.flush();
    };
}

#[no_mangle]
pub extern "C" fn sys_open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let res = REAL_OPEN(path, oflag, mode);
    if res > 0 {
        record_path(path);
    }
    res
}

// FIXME, we might not need this
#[no_mangle]
pub extern "C" fn sys_openat(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> c_int {
    let res = REAL_OPENAT(dirfd, pathname, flags, mode);
    if res >= 0 && dirfd == libc::AT_FDCWD {
        record_path(pathname);
    }
    res
}

#[no_mangle]
pub extern "C" fn sys_open64(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let res = REAL_OPEN64(path, oflag, mode);
    if res > 0 {
        record_path(path);
    }
    res
}

// FIXME, we might not need this
#[no_mangle]
pub extern "C" fn sys_openat64(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> c_int {
    let res = REAL_OPENAT64(dirfd, pathname, flags, mode);
    if res >= 0 && dirfd == libc::AT_FDCWD {
        record_path(pathname);
    }
    res
}
