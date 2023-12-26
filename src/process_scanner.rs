use std::{ fs, fmt };
use std::collections::HashMap;
use regex::Regex;

pub struct ProcessScanner {
    processes: HashMap<u32, (u32, String)>
}

impl ProcessScanner {
    pub fn new() -> Self {
        let pids = ProcessScanner::get_pids();
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
}

impl fmt::Display for ProcessScanner {
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
