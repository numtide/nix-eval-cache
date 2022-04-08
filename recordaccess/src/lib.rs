use ctor::{ctor, dtor};
use lazy_static::lazy_static;
use libc::{c_char, c_int, mode_t};
use nix::unistd;

use std::collections::HashSet;
use std::ffi::{CString, CStr};
use std::env;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::FromRawFd;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;

static STATE: AtomicUsize = AtomicUsize::new(0);
static REPORT_FD: AtomicI32 = AtomicI32::new(0);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;
const FINISHED: usize = 3;

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
}

#[ctor]
fn init() {
    if STATE
        .compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
        .is_err()
    {
        return;
    }

    // Should we unset NIX_EVAL_CACHE_FD in childs?
    match env::var("NIX_EVAL_CACHE_FD") {
        Ok(strval) => {
            if let Ok(val) = strval.parse::<i32>() {
                REPORT_FD.store(val, Ordering::Release)
            }
            STATE.store(INITIALIZED, Ordering::Release);
        }
        Err(_) => {
            return;
        }
    };
}

pub fn record_path(path: *const c_char) {
    let c_str: &CStr = unsafe { CStr::from_ptr(path) };
    PATHS.lock().unwrap().insert(c_str.to_owned());
}

#[no_mangle]
pub extern "C" fn sys_open(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    record_path(path);
    REAL_OPEN(path, oflag, mode)
}

// FIXME, we might not need this
#[no_mangle]
pub extern "C" fn sys_openat(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> c_int {
    if dirfd == libc::AT_FDCWD {
        record_path(pathname);
    }
    REAL_OPENAT(dirfd, pathname, flags, mode)
}

#[no_mangle]
pub extern "C" fn sys_open64(path: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    record_path(path);
    REAL_OPEN64(path, oflag, mode)
}

// FIXME, we might not need this
#[no_mangle]
pub extern "C" fn sys_openat64(
    dirfd: c_int,
    pathname: *const c_char,
    flags: c_int,
    mode: mode_t,
) -> c_int {
    if dirfd == libc::AT_FDCWD {
        record_path(pathname);
    }
    REAL_OPENAT64(dirfd, pathname, flags, mode)
}

#[dtor]
fn deinit() {
    if STATE
        .compare_exchange(INITIALIZED, FINISHED, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    let fd = REPORT_FD.load(Ordering::Acquire);
    let mut f = unsafe { File::from_raw_fd(fd) };
    let paths = PATHS.lock().unwrap();
    for p in paths.iter() {
        //use std::slice;
        //use std::str;
        //let s = unsafe { str::from_utf8_unchecked(slice::from_raw_parts(p.as_ptr() as *const u8, libc::strlen(p.as_ptr())+1)) };
        //println!("{}", s);
        let _ = f.write_all(p.as_bytes());
        let _ = f.write_all(b"\0");
    }
    let _ = f.flush();
}
