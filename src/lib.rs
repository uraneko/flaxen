pub mod commissioner;
pub mod container;
pub mod events;
pub mod input;
pub mod kbd_decode;
pub mod presets;
pub mod raw_mode;
pub mod styled;
pub mod termbuf;

pub use kbd_decode::*;
pub(crate) use raw_mode::*;
use termbuf::*;

use std::ops::Range;

// TODO: remove Terminal struct altogether
// buffer is a winsize method
// cursor, raw and sol can be globals
// also, init commissioner

use container::ID;

#[derive(Debug, Default)]
struct LayerTree<'a, 'b> {
    tree: Layer0<'a, 'b>,
}

#[derive(Debug, Default)]
struct Layer0<'a, 'b> {
    value: Vec<ID<'static>>,
    components: Vec<Layer1<'a, 'b>>,
    link_less: Vec<LinkLess>,
}

// components are on this layer
#[derive(Debug, Default)]
struct Layer1<'a, 'b>
where
    'a: 'b,
{
    value: Vec<ID<'a>>,
    items: Vec<Layer2<'b>>,
}

// text types can be found on this layer
#[derive(Debug, Default)]
struct Layer2<'a> {
    value: Vec<ID<'a>>,
}

// types not part of the object heirarchy but still have an id can be found here
// eg. StyleGraphs and Styles
#[derive(Debug, Default)]
struct LinkLess {
    value: Vec<ID<'static>>,
}

impl<'a, 'b> LayerTree<'a, 'b> {
    fn new() -> Self {
        Self {
            tree: Layer0::default(),
        }
    }
}

use crate::container::Point;

// if feature 'presets' is on then the crate only compiles the basic presets,
// in this case, cache is only a vec of History values that get saved to a file,
// otherwise cache is a libsql db that can house many caches, not just input Histories
#[derive(Debug)]
pub struct Term<'a, 'b> {
    id: ID<'static>,
    cache: HashMap<&'static str, Vec<u8>>,
    buf: Vec<u8>,
    width: u16,
    height: u16,
    cursor: Point<u16>,
    tree: LayerTree<'a, 'b>,
}

impl<'a, 'b> Term<'a, 'b> {
    pub fn new() -> Self {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, 0);

        Self {
            id: "T0",
            cache: HashMap::new(),
            cursor: Point::new(0, 0),
            width: ws.cols(),
            height: ws.rows(),
            buf,
            tree: LayerTree::<'a, 'b>::new(),
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
pub trait SpaceMorph {
    fn rescale(&mut self);
}

#[derive(Debug)]
struct DB {}
#[derive(Debug)]
struct Memory {}
use std::collections::{BTreeMap, HashMap};

use std::io::StdoutLock;
use std::io::Write;

impl<'a, 'b> Term<'a, 'b> {
    fn f(&mut self, x: u16, y: u16) {
        let esc_seq = format!("\x1b{};{}f", x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::Term;

    #[test]
    fn test_raw_mode() {
        let _ = Term::new();

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
