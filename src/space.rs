use crate::object_tree::{Container, ObjectTree, Term, Text};

use std::collections::HashMap;
use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, RemAssign, ShlAssign, ShrAssign, Sub,
        SubAssign,
    },
};

// top, right, bottom, left
// a is the new object
// b is the pre existing one
pub fn area_conflicts(
    ax0: u16,
    ay0: u16,
    aw: u16,
    ah: u16,
    bx0: u16,
    by0: u16,
    bw: u16,
    bh: u16,
) -> [i16; 4] {
    [
        by0 as i16 - ay0 as i16 + ah as i16, // bottom,  if > 0 then no conflict
        by0 as i16 + bh as i16 - ay0 as i16, // top,    if < 0 then no conflict
        bx0 as i16 - ax0 as i16 + aw as i16, // right, if > 0 then no conflict
        bx0 as i16 + bw as i16 - ax0 as i16, // left, if < 0 then no conflic
    ]
}

pub fn between<T: std::cmp::PartialOrd>(a: T, b: T, c: T) -> bool {
    a < b && b < c
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Border {
    #[default]
    None,
    Uniform(char),
    Polyform {
        rcorner: char,
        lcorner: char,
        tcorner: char,
        bcorner: char,
        rl: char,
        tb: char,
    },
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Padding {
    #[default]
    None,

    Inner {
        top: u16,
        bottom: u16,
        right: u16,
        left: u16,
    },

    Outer {
        top: u16,
        bottom: u16,
        right: u16,
        left: u16,
    },

    InOut {
        inner_top: u16,
        inner_bottom: u16,
        inner_right: u16,
        inner_left: u16,
        outer_top: u16,
        outer_bottom: u16,
        outer_right: u16,
        outer_left: u16,
    },
}

impl Term {
    pub fn rescale(&mut self, wdiff: u16, hdiff: u16) {
        self.w *= wdiff;
        self.h *= hdiff;
        self.cx *= wdiff;
        self.cy *= hdiff;
    }
}

impl Container {
    pub fn rescale(&mut self, wnew: u16, hnew: u16) {
        let [wdiff, hdiff] = [wnew / self.w, hnew / self.h];
        self.w *= wdiff;
        self.h *= hdiff;
        self.x0 *= wdiff;
        self.y0 *= hdiff;
    }
}

use std::cmp::Ordering;

impl Text {
    pub fn rescale(&mut self, wnew: u16, hnew: u16) {
        let [wdiff, hdiff] = [wnew / self.w, hnew / self.h];
        self.w *= wdiff;
        self.h *= hdiff;
        self.x0 *= wdiff;
        self.y0 *= hdiff;

        if self.cx >= self.w {
            self.cx = self.w - 1
        }
        if self.cy >= self.h {
            self.cy = self.h - 1
        }

        let diff = self.value.len() as isize - (self.w * self.h) as isize;
        match diff.cmp(&0) {
            // unchanged, shouldn't be possible since we had a window resize
            Ordering::Equal => (),
            // window size shrank
            Ordering::Greater => (0..diff).into_iter().for_each(|_| {
                self.value.remove(self.value.len());
            }),

            // window size grew
            Ordering::Less => (0..diff).into_iter().for_each(|_| {
                self.value.push(None);
            }),
        }
    }
}

// NOTE: window resizing is polled at every frame redraw
// logic for when the window is resized:
// first the resize is detected by a buffer image events
// then the buffer tells the commissioner
// the commissioner in turn tells every component and its items in the buffer
// then he rescales their rendered data to the new window scale and dimensions
// then the buffer updates its buf with the new stuff and updates its global cursor
pub trait SpaceAwareness {
    // responds to window resize events
    fn rescale(&mut self, wdiff: u16, hdiff: u16);
}
