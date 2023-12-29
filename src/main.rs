use std::io::Error as io_err;
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::{ Sender, Receiver };

mod process_scanner;
use process_scanner::ProcessScanner;

mod inotify_api;
use crate::inotify_api::{ inotify_init1, read };
mod filesystem_watcher;
use filesystem_watcher::FileSystemWatcher;


fn main() -> Result<(), Box<dyn Error>> {
    let mut processes = ProcessScanner::new();
    println!("{}", processes);

    let mut fsw = FileSystemWatcher::new();
    fsw.walk_directories(vec![String::from("/opt"), String::from("/usr")]);
    fsw.add_watch();

    let (tx,rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    
    FileSystemWatcher::observe(fsw, tx);

    loop {
        rx.recv();
        processes.refresh();
    }

    Ok(()) 
}
