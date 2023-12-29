use std::{ fs, fmt };
use std::collections::HashMap;
use regex::Regex;

#[derive(Debug)]
struct Process {
    pid: u32,
    euid: u32,
    cmdline: String,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PID: {0: <6} | EUID: {1: <6} | cmd: {2: <10}",
            self.pid,
            self.euid,
            self.cmdline,
        );
        Ok(())
    }
}

pub struct ProcessScanner {
    processes: HashMap<u32, Process>
}

impl ProcessScanner {
    pub fn new() -> Self {
        let pids = ProcessScanner::get_pids();
        let mut processes = HashMap::new();

        for p in pids {
            if let Some(process) = ProcessScanner::get_process_metadata(p) {
                processes.insert(p, process);
            }
        }

        ProcessScanner {
            processes
        }
    }

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

    fn get_process_metadata(pid: u32) -> Option<Process> {
        let status = match std::fs::read_to_string(format!("{}{}{}", "/proc/", pid, "/status")) {
            Ok(s) => s,
            Err(_) => String::from("")
        };

        let mut cmdline = match std::fs::read_to_string(format!("{}{}{}", "/proc/", pid, "/cmdline")) {
            Ok(c) => c.trim().replace("\0", " ").replace("\n", "").replace("\r", ""),
            Err(_) => String::from("???")
        };

        let re = Regex::new(r"Uid:\t\d+\t\d+\t(?P<uid>\d+)").unwrap();
        let mut euid: u32 = 0;
        for caps in re.captures_iter(&status) {
            euid = caps["uid"].parse().unwrap();
        }
        
        Some(Process {
            pid,
            euid,
            cmdline,
        })
    }

    pub fn refresh(&mut self) {
        let pids = ProcessScanner::get_pids();

        for p in pids {
            if let None = self.processes.get(&p) {
                if let Some(process) = ProcessScanner::get_process_metadata(p) {
                    self.processes.insert(p, process);
                    println!("{}", self.processes.get(&p).unwrap());
                }
            }
        }
        
    }
}

impl fmt::Display for ProcessScanner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (pid, process) in self.processes.iter() {
            write!(f, "{}\n", process);
        }
        Ok(())
    }
}
