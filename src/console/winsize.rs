// use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
//
// fn main() {
//     unsafe {
//         let mut ws: winsize = std::mem::zeroed();
//         let res = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);
//         print!("{:?}", ws);
//     }
// }

// from /usr/include/sys/ioctl.h
#[link(name = "c")]
extern "C" {
    fn ioctl(__fd: i32, __request: u64, ...) -> i32;
}

/// ffi bindings for c struct from '/usr/include/asm-generic/termios.h'
#[derive(Debug, Default)]
#[repr(C)]
pub struct winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

// from /usr/include/unistd.h
const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;
const STDERR_FILENO: i32 = 2;

// from /usr/include/asm-generic/ioctl.h
const TIOCGWINSZ: u64 = 0x5413;
const TIOCSWINSZ: u64 = 0x5414;

impl winsize {
    /// creates a new winsize instance with the current window width and height
    pub fn from_ioctl() -> Self {
        let mut ws = Default::default();
        _ = unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws) };

        ws
    }

    /// returns the width of the terminal window from this winsize instane
    pub fn cols(&self) -> u16 {
        self.ws_col
    }

    /// returns the height of the terminal window from this winsize instane
    pub fn rows(&self) -> u16 {
        self.ws_row
    }

    pub fn resized(&mut self) -> bool {
        let ws = Self::from_ioctl();
        if self.cols() != ws.cols() || self.rows() != ws.rows() {
            *self = ws;

            return true;
        }

        false
    }
}
