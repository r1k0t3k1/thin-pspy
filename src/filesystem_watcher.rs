use std::path::Path;
use std::io::Error;
use std::ffi::CString;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use crate::inotify_api;

#[derive(Clone)]
pub struct FileSystemWatcher {
    // TODO
    fd: i32,
    max_user_watches: u32,
    pub root_directories: Vec<String>,
    pub watch_directories: Vec<String>,
}

impl FileSystemWatcher {
    pub fn new() -> Self {
       let max_user_watches = std::fs::read_to_string("/proc/sys/fs/inotify/max_user_watches")
           .unwrap()
           .trim()
           .parse::<u32>()
           .unwrap();
       FileSystemWatcher {
           fd: 0,
           max_user_watches,
           root_directories: vec![],
           watch_directories: vec![],
       }
    }

    pub fn walk_directories(&mut self, directories: Vec<String>) {
        if !directories.iter().all(|d| Path::new(d).exists()) {
            // TODO
            println!("not exists.");
        }
        self.root_directories = directories; 

        for rd in &self.root_directories {
            walk_directory(Path::new(&rd), &mut self.watch_directories, 3);
        }
        self.watch_directories.sort();
    }

    pub fn add_watch(&mut self) {
        let fd = unsafe { inotify_api::inotify_init1(inotify_api::IN_CLOEXEC) };
        if fd == -1 {
            eprintln!("failed in inotify_init1. fd = {}, last OS Error = {}", fd, Error::last_os_error());
            std::process::exit(1);
        }
        self.fd = fd;

        for d in &self.watch_directories {
            let watch_fd = unsafe {
                inotify_api::inotify_add_watch(
                    fd,
                    CString::new(d.clone()).unwrap().as_ptr(),
                    inotify_api::IN_ALL_EVENTS,
                )
            };
            if watch_fd == -1 {
                eprintln!(
                    "[ERR] inotify_add_watch \"{}\". watch_fd = {}, last OS Error = {}",
                    d,
                    watch_fd,
                    Error::last_os_error()
                );
            }
    
        }
    }

    pub fn observe(fsw: Self, sender: Sender<()>) {
        let fd = fsw.fd;
        let t = thread::spawn(move|| {
            loop {
                let mut buf = [0_u8;1024];
                let len = unsafe { inotify_api::read(fd, buf.as_mut_ptr() as *mut u8, buf.len()) };
                // inotify_init1でIN_NON_BLOCKを渡しているため即EAGAINが返る。
                if len == -1 {
                    if let Some(err) = Error::last_os_error().raw_os_error() {
                        if err == 22 {
                            println!("{}", Error::last_os_error());
                        }
                    };
                    continue;
                }
                sender.send(()).unwrap();
            }
        });
    }
}

fn walk_directory(directory: &Path, directory_list: &mut Vec<String>, depth: u8) {
    if depth == 0 { return; }
    directory_list.push(directory.to_str().unwrap().to_string());

    if let Ok(d) =  std::fs::read_dir(directory) {
        for entry in d {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directory_list.push(path.to_str().unwrap().to_string());
                walk_directory(&path, directory_list, depth-1);
            }
        }
    }
}
