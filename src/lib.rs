#![allow(warnings)]
pub mod commissioner;
pub mod container;
pub mod events;
pub mod history;
pub mod input;
pub mod kbd_decode;
pub mod presets;
pub mod raw_mode;
pub mod styles;
pub mod termbuf;

pub use kbd_decode::*;
pub(crate) use raw_mode::*;
use termbuf::*;

use std::ops::Range;

// TODO: remove Terminal struct altogether
// buffer is a winsize method
// cursor, raw and sol can be globals
// also, init commissioner

// TODO: change ID to be a u16 value
// the first byte would describe what the type is; eg, Events, Themes, Terms...
// the second byte would describe the path of the ID holder; eg, Term0 -> Container2 -> TextInput5

use container::ID;

use crate::container::Point;

// if feature 'presets' is on then the crate only compiles the basic presets,
// in this case, cache is only a vec of History values that get saved to a file,
// otherwise cache is a libsql db that can house many caches, not just input Histories
#[derive(Debug)]
pub struct Term<'a, 'b, const CLASS: char> {
    overlaying: bool,
    id: ID<'static>,
    cache: HashMap<&'static str, Vec<u8>>,
    buf: Vec<u8>,
    width: u16,
    height: u16,
    cursor: Point<u16>,
    tree: ObjectTree<'a, 'b, CLASS>,
}

// TODO: shelve the layer tree stuff in favor of inluding the whole object tree inside the term

use crate::container::Container;

use crate::styles::StyleStrategy;

#[derive(Debug)]
struct ObjectTree<'a, 'b, const CLASS: char> {
    containers: Vec<Container<'a, 'b, CLASS>>,
    linkless: Vec<StyleStrategy>,
}

impl<'a, 'b, const CLASS: char> ObjectTree<'a, 'b, CLASS> {
    fn new() -> Self {
        Self {
            containers: vec![],
            linkless: vec![],
        }
    }
}

impl<'a, 'b, const CLASS: char> Term<'a, 'b, CLASS> {
    pub fn new() -> Self {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, 0);

        Self {
            id: "T0",
            overlaying: false,
            cache: HashMap::new(),
            cursor: Point::new(0, 0),
            width: ws.cols(),
            height: ws.rows(),
            buf,
            tree: ObjectTree::<'a, 'b, CLASS>::new(),
        }
    }

    pub fn print_buf(&self) {
        for idxr in 0..self.buf.len() / self.width as usize {
            print!("\r\n");
            for idxc in 0..self.buf.len() / self.height as usize {
                print!("{}", self.buf[idxr]);
            }
        }
    }

    fn refresh(&mut self) {
        let ws = winsize::from_ioctl();
        self.width = ws.cols();
        self.height = ws.rows();
    }

    pub fn j(&mut self, p: u8) {
        let esc_seq = format!("\x1b[{}J", p);
    }
}

// NOTE: window resizing is polled at every frame redraw
// logic for when the window is resized,
// first the resize is detected by a buffer image events
// then the buffer tells the commissioner
// the commissioner in turn tells every component and its items in the buffer
// then he rescales their rendered data to the new window scale and dimensions
// then the buffer updates its buf with the new stuff and updates its global cursor
pub trait SpaceAwareness {
    fn rescale(&mut self);
}

// TODO:
// change ID from using &str and the current
// slew of lifetimes i have to write to using u8 values as IDs,
// the higher four bits on the u8 are used to identify the type being ided,
// ie, Event, Container, Text...
// the lower four bits are that exact instance's identification values

#[derive(Debug)]
struct DB {}
#[derive(Debug)]
struct Memory {}
use std::collections::{BTreeMap, HashMap};

use std::io::StdoutLock;
use std::io::Write;

impl<'a, 'b, const CLASS: char> Term<'a, 'b, CLASS> {
    fn f(&mut self, x: u16, y: u16) {
        let esc_seq = format!("\x1b{};{}f", x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::Term;

    #[test]
    fn test_raw_mode() {
        let _ = Term::<'w'>::new();

        println!("we are now inside raw mode");
        println!("we are now inside raw mode");
    }
}

// TODO:
// 1 => raw mode + alternate screen + winsize + term buffer of NUL... done
// 2 => kbd read + decode utf8... wip
// 3 => styled... wip... needs modifications
// 4 => event queue ... wip
// 5 containers... stalled
// 5a => inner input logic
// 5b => non editable text container logic (including prompt)
// 5c => popup container logic
// 6 => panes support

use commissioner::Commissioner;
