use std::fmt;

extern "C" {
    pub fn inotify_init1(flags: i32) -> i32;
    pub fn inotify_add_watch(fd: i32, pathname: *const i8, mask: u32) -> i32;
    pub fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
}

pub const IN_CLOEXEC: i32 = 524288;
pub const IN_NONBLOCK: i32 = 2048;
pub const IN_ALL_EVENTS: u32 = 4095;

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
    pub fn new(mask: u32) -> Option<Self> {
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
    pub fn new(buf: &[u8;1024]) -> Self {
        let mut v = buf[16..].to_vec();
        v.retain(|x| *x != 0);
        inotify_event {
           wd: i32::from_le_bytes(buf[0..4].try_into().unwrap()),
           mask: Mask::new(u32::from_le_bytes(buf[4..8].try_into().unwrap())).unwrap(),
           cookie: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
           len: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
           name: String::from(""), // TODO
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

