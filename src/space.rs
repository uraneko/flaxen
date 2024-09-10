use crate::object_tree::{Container, ObjectTree, Term, Text};

use std::collections::HashMap;
use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, RemAssign, ShlAssign, ShrAssign, Sub,
        SubAssign,
    },
};

// FIXME: this is partially wrong but the wrong part is needed for the overlay only
// fix this before making the overlay
// top, right, bottom, left
// a is the new object
// b is the pre existing one
pub fn area_conflicts(
    newx0: u16,
    newy0: u16,
    neww: u16,
    newh: u16,
    oldx0: u16,
    oldy0: u16,
    oldw: u16,
    oldh: u16,
) -> [i16; 4] {
    //     println!(
    //         "
    //         top = ({newy0} + {newh})  - {oldy0} = {},
    //         right = {newx0}  - ({oldx0} + {oldw}) = {},
    //         bottom = {newy0}  - ({oldy0} + {oldh}) = {},
    //         left = ({newx0} + {neww})  - {oldx0} = {},
    // ",
    //         (newy0 + newh) as i16 - oldy0 as i16,
    //         newx0 as i16 - (oldx0 + oldw) as i16,
    //         newy0 as i16 - (oldy0 + oldh) as i16,
    //         (newx0 + neww) as i16 - oldx0 as i16,
    //     );
    [
        // the new object is to the top of the old one
        // < 0 means the new area is valid
        if newy0 > oldy0 { -1 } else { 1 } * (newy0 + newh) as i16 - oldy0 as i16,
        // the new object is to the right of the old one
        // > 0 means the new area is valid
        if newx0 > oldx0 { 1 } else { -1 } * newx0 as i16 - (oldx0 + oldw) as i16,
        // the new area is to the bottom of the old one
        // > 0 means the new area is valid
        if newy0 > oldy0 { 1 } else { -1 } * newy0 as i16 - (oldy0 + oldh) as i16,
        // the new object is to the left of the old one
        // < 0 means the new area is valid
        if newx0 > oldx0 { -1 } else { 1 } * (newx0 + neww) as i16 - oldx0 as i16,
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
