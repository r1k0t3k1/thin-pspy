use std::{ slice, fs, fmt };
use std::ffi::CString;
use std::path::Path;
use std::error::Error;
use std::io::Error as io_err;
use std::io::ErrorKind;

use std::collections::HashMap;
use regex::Regex;

extern "C" {
    fn inotify_init1(flags: i32) -> i32;
    fn inotify_add_watch(fd: i32, pathname: *const i8, mask: u32) -> i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
}

const IN_CLOEXEC: i32 = 524288;
const IN_NONBLOCK: i32 = 2048;
const IN_ALL_EVENTS: u32 = 4095;

#[derive(PartialEq,Debug)]
pub enum Mask {
    IN_ACCESS = 0x1,     
    IN_MODIFY = 0x2,
    IN_ATTRIB = 0x4,     
    IN_CLOSE_WRITE = 0x8,     
    IN_CLOSE_NOWRITE = 0x10,     
    IN_CLOSE = 0x8 | 0x10,
    IN_OPEN = 0x20,     
    IN_MOVED_FROM = 0x40,
    IN_MOVED_TO = 0x80,
    IN_MOVE = 0x40 | 0x80,
    IN_CREATE = 0x100,
    IN_DELETE = 0x200,
    IN_DELETE_SELF = 0x400,
    IN_MOVE_SELF = 0x800,
    Undefined = 0x1000,
}

impl Mask {
    fn new(mask: u32) -> Option<Self> {
        match mask {
            0x1         => Some(Mask::IN_ACCESS),
            0x2         => Some(Mask::IN_MODIFY),
            0x4         => Some(Mask::IN_ATTRIB),     
            0x8         => Some(Mask::IN_CLOSE_WRITE),     
            0x10        => Some(Mask::IN_CLOSE_NOWRITE),     
            0x8 | 0x10  => Some(Mask::IN_CLOSE),
            0x20        => Some(Mask::IN_OPEN),     
            0x40        => Some(Mask::IN_MOVED_FROM),
            0x80        => Some(Mask::IN_MOVED_TO),
            0x40 | 0x80 => Some(Mask::IN_MOVE),
            0x100       => Some(Mask::IN_CREATE),
            0x200       => Some(Mask::IN_DELETE),
            0x400       => Some(Mask::IN_DELETE_SELF),
            0x800       => Some(Mask::IN_MOVE_SELF),
            _           => Some(Mask::Undefined),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct inotify_event {
    pub wd: i32,
    pub mask: Mask,
    pub cookie: u32,
    pub len: u32,
    pub name: String,
}

impl inotify_event {
    fn new(buf: &[u8;1024]) -> Self {
        let mut v = buf[16..].to_vec();
        v.retain(|x| *x != 0);
        inotify_event {
           wd: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
           mask: Mask::new(u32::from_le_bytes(buf[4..8].try_into().unwrap())).unwrap(),
           cookie: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
           len: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
           name: String::from_utf8(v).unwrap(),
        }
    }
}

impl fmt::Display for inotify_event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "wd = {}, mask = {:?}, cookie = {}, len = {}, name = {}",
            self.wd,
            self.mask,
            self.cookie,
            self.len,
            self.name,
        )
    }
}

struct ProcessHash {
    processes: HashMap<u32, (u32, String)>
}

impl ProcessHash {
    fn get_pids() -> Vec<u32> {
        let entries = fs::read_dir("/proc").unwrap();
        let mut pids = vec![];
        for entry in entries.into_iter() {
            let pid = entry.unwrap().file_name().into_string().unwrap();
            if pid.parse::<u32>().is_ok() {
                pids.push(pid.parse().unwrap());
            }
        }
        pids
    }

    fn new() -> Self {
        let pids = ProcessHash::get_pids();
        let mut processes = HashMap::new();

        for p in pids {
            let status = std::fs::read_to_string(format!("{}{}{}", "/proc/", p, "/status")).unwrap();
            let re = Regex::new(r"Uid:\t\d+\t\d+\t(?P<uid>\d+)").unwrap();
            let mut euid: u32 = 0;
            for caps in re.captures_iter(&status) {
                euid = caps["uid"].parse().unwrap();
            }
            let mut cmdline = std::fs::read_to_string(format!("{}{}{}", "/proc/", p, "/cmdline")).unwrap();
            cmdline = cmdline.trim().replace("\0", " ");
            processes.insert(p, (euid, cmdline));
        }

        ProcessHash {
            processes
        }
    }
}

impl fmt::Display for ProcessHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (pid, metadata) in self.processes.iter() {
            write!(
                f,
                "PID: {0: <6} | EUID: {1: <6} | cmd: {2: <10}\n",
                pid,
                metadata.0,
                metadata.1,
            );
        }
        Ok(())
    }
}

fn walk_dir(dir: &Path, dir_list: &mut Vec<String>, depth: u8) {
    if depth == 0 { return; }
    dir_list.push(dir.to_str().unwrap().to_string());

    if let Ok(d) =  fs::read_dir(dir) {
        for entry in d {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dir_list.push(path.to_str().unwrap().to_string());
                walk_dir(&path, dir_list, depth-1);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let max_user_watches = std::fs::read_to_string("/proc/sys/fs/inotify/max_user_watches")?.trim().parse::<u32>()?;

    let fd = unsafe { inotify_init1(IN_CLOEXEC) };
    if fd == -1 {
        eprintln!("failed in inotify_init1. fd = {}, last OS Error = {}", fd, io_err::last_os_error());
        std::process::exit(1);
    }

    let pathname = CString::new("/home/rikoteki/Desktop/inotify_test.txt").expect("pathname init failed.");
    let mut dirs = vec![];
    walk_dir(&Path::new("/opt"), &mut dirs, 3);
    dirs.sort();
    println!("{:?}", dirs);    

    for d in dirs {
        let watch_fd = unsafe { 
            inotify_add_watch(fd, CString::new(d.clone()).unwrap().as_ptr(), Mask::IN_OPEN as u32) 
        };
        if watch_fd == -1 {
            eprintln!(
                "[ERR] inotify_add_watch \"{}\". watch_fd = {}, last OS Error = {}",
                d,
                watch_fd,
                io_err::last_os_error()
            );
            //std::process::exit(2);
        }
    }

    let processes = ProcessHash::new();
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
        let event = inotify_event::new(&buf);
        println!("{}", event);
    }
        

    Ok(()) 
}
