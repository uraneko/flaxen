pub mod container;
pub mod events;
pub mod kbd_decode;
pub mod raw_mode;
pub mod styled;
pub mod termbuf;

pub mod shin_sekai;

pub use kbd_decode::*;
pub(crate) use raw_mode::*;
use termbuf::*;

use shin_sekai::*;

#[derive(Debug, Default)]
struct Point {
    x: u16,
    y: u16,
}

impl Point {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default)]
pub struct Terminal {
    buffer: Vec<u16>,
    winsize: winsize,
    cursor: Point,
    raw: termios,
    // esc_seq: String,
    sol: Option<StdoutLock<'static>>,
}

impl Terminal {
    pub fn new() -> Self {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, 0);

        let mut sol = std::io::stdout().lock();
        // _ = sol.write(b"\x1b[?1049h");
        // _ = sol.write(b"\x1b[0;0f");
        // _ = sol.flush();

        Self {
            buffer: buf,
            winsize: ws,
            cursor: Default::default(),
            raw: raw_mode(),
            sol: Some(sol),
        }
    }

    pub fn j(&mut self, p: u8) {
        let esc_seq = format!("\x1b[{}J", p);
        if self.sol.is_some() {
            _ = self.sol.as_mut().unwrap().write(&esc_seq.as_bytes());
        }
    }
}

use std::io::StdoutLock;
use std::io::Write;

impl Drop for Terminal {
    fn drop(&mut self) {
        print!("cleaning up... ",);

        cooked_mode(&self.raw);
        if self.sol.is_some() {
            _ = self.sol.as_mut().unwrap().write(b"\x1b[?1049l");
        }

        print!("  done");
    }
}

struct ScreenShot<'a> {
    buffer: &'a [u16],
    rows: u16,
    cols: u16,
    origin: Point,
}

impl Terminal {
    fn f(&mut self, x: u16, y: u16) {
        let esc_seq = format!("\x1b{};{}f", x, y);
        if self.sol.is_some() {
            _ = self.sol.as_mut().unwrap().write(&esc_seq.as_bytes());
        }
    }

    // breaks the program
    pub fn c(&mut self) {
        if self.sol.is_some() {
            _ = self.sol.as_mut().unwrap().write(b"\x1b[c");
        }
    }

    fn write(&mut self, input: &[u8]) {
        if self.sol.is_some() {
            _ = self.sol.as_mut().unwrap().write(input);
        }
    }

    fn screenshot(&self) -> ScreenShot {
        ScreenShot {
            buffer: &self.buffer,
            rows: self.winsize.rows(),
            cols: self.winsize.cols(),
            origin: Point::new(0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Terminal;

    #[test]
    fn test_raw_mode() {
        let _ = Terminal::new();

        println!("we are now inside raw mode");
        println!("we are now inside raw mode");
    }
}

// TODO:
// 1 => raw mode + alternate screen + winsize + term buffer of NUL... done
// 2 => kbd read + decode utf8... wip
// 3 => styled... done... needs modifications
// 4 => event queue ... wip
// 5 containers... stalled
// 5a => inner input logic
// 5b => non editable text container logic (including prompt)
// 5c => popup container logic
