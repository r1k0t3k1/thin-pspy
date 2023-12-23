use std::ffi::CString;
use std::io::Error;

extern "C" {
    fn inotify_init1(flags: i32) -> i32;
    fn inotify_add_watch(fd: i32, pathname: *const i8, mask: u32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
}

const IN_NONBLOCK: i32 = 2048;
const IN_ALL_EVENTS: u32 = 4095;

fn main() {
    let fd = unsafe { inotify_init1(IN_NONBLOCK) };
    if fd == -1 {
        eprintln!("failed in inotify_init1. fd = {}, last OS Error = {}", fd, Error::last_os_error());
        std::process::exit(1);
    }

    let pathname = CString::new("/home/rikoteki/Desktop/inotify_test.txt").expect("pathname init failed.");

    let watch_fd = unsafe { inotify_add_watch(fd, pathname.as_ptr(), IN_ALL_EVENTS) };
    if watch_fd == -1 {
        eprintln!("failed in inotify_add_watch. watch_fd = {}, last OS Error = {}", watch_fd, Error::last_os_error());
        std::process::exit(2);
    }

    let mut buf = [0_u8;4096];
    loop {
        let len = unsafe { read(fd, buf.as_mut_ptr() as *mut u8, buf.len()) };
        // inotify_init1でIN_NON_BLOCKを渡しているため即EAGAINが返る。
        if len == -1 {
            continue;
        }
        println!("{}", len);
    }
    
}
