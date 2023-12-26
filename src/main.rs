use std::io::Error as io_err;
use std::error::Error;

mod process_scanner;
use process_scanner::ProcessScanner;

mod inotify_api;
use crate::inotify_api::{ inotify_init1, read };
mod filesystem_watcher;
use filesystem_watcher::FileSystemWatcher;


fn main() -> Result<(), Box<dyn Error>> {

    let mut fsw = FileSystemWatcher::new();
    fsw.walk_directories(vec![String::from("/opt")]);
    let fd = fsw.add_watch();

    let processes = ProcessScanner::new();
    println!("{}", processes);

    let mut buf = [0_u8;1024];
    loop {
        let len = unsafe { read(fd, buf.as_mut_ptr() as *mut u8, buf.len()) };
        // inotify_init1でIN_NON_BLOCKを渡しているため即EAGAINが返る。
        if len == -1 {
            if let Some(err) = io_err::last_os_error().raw_os_error() {
                if err == 22 {
                    println!("{}", io_err::last_os_error());
                }
            };
            continue;
        }
        let event = inotify_api::inotify_event::new(&buf);
        println!("{}", event);
    }

    Ok(()) 
}
