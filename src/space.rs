use crate::components::{ComponentTree, Container, Term, Text};

use std::collections::HashMap;
use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, RemAssign, ShlAssign, ShrAssign, Sub,
        SubAssign,
    },
};

pub(crate) fn area_conflicts(
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

pub(crate) fn between<T: std::cmp::PartialOrd>(a: T, b: T, c: T) -> bool {
    a < b && b < c
}

// this exists because the border doesn't have a width and height in this lib, it is always a
// single cell box surrounding the value
pub(crate) fn border_fit(border: &Border, padding: &Padding, w: u16, h: u16) -> bool {
    if let Border::Manual {
        t0,
        t1,
        l0,
        l1,
        r0,
        r1,
        b0,
        b1,
        ..
    } = border
    {
        // extract the inner padding values if they exist
        let [pit, pir, pil, pib] = if let Padding::Inner {
            top,
            bottom,
            left,
            right,
        } = padding
        {
            [top, right, left, bottom]
        } else if let Padding::InOut {
            inner_top,
            inner_right,
            inner_bottom,
            inner_left,
            ..
        } = padding
        {
            [inner_top, inner_right, inner_left, inner_bottom]
        } else {
            [&0u16; 4]
        };

        // border size validity checks
        if pit + pib + h < (l0.chars().count() + l1.chars().count()) as u16
            || pit + pib + h < (r0.chars().count() + r1.chars().count()) as u16
            || pir + pil + w < (t0.chars().count() + t1.chars().count()) as u16
            || pir + pil + w < (b0.chars().count() + b1.chars().count()) as u16
        {
            return false;
        }
    }

    true
}

/// Container and Text objects border
#[derive(Debug, Default, Clone, Copy)]
pub enum Border {
    /// no border
    #[default]
    None,
    /// same character border
    Uniform(char),
    /// border with different chars for the corners and the sides
    Polyform {
        /// top right corner border char
        trcorner: char,
        /// top left corner border char
        tlcorner: char,
        /// bottom left corner border char
        blcorner: char,
        /// bottom right corner border char
        brcorner: char,
        /// right/left sides border char
        rl: char,
        /// top/bottom sides border char
        tb: char,
    },
    Manual {
        /// top left corner
        tlcorner: char,
        /// top right corner
        trcorner: char,
        /// bottom right corner
        brcorner: char,
        /// bottom left corner
        blcorner: char,

        // top side
        t0: &'static str,
        tp: char,
        t1: &'static str,

        // right side
        r0: &'static str,
        rp: char,
        r1: &'static str,

        //
        l0: &'static str,
        lp: char,
        l1: &'static str,

        b0: &'static str,
        bp: char,
        b1: &'static str,
    },
}

/// Container and Text objects padding space
#[derive(Debug, Default, Clone, Copy)]
pub enum Padding {
    /// no padding
    #[default]
    None,

    /// padding only between the value inside the object and its border
    Inner {
        /// top side padding
        top: u16,
        /// bottom side padding
        bottom: u16,
        /// right side padding
        right: u16,
        /// left side padding
        left: u16,
    },

    /// padding only around the border of the object
    Outer {
        /// top side padding
        top: u16,
        /// bottom side padding
        bottom: u16,
        /// right side padding
        right: u16,
        /// left side padding
        left: u16,
    },

    /// padding both around the border and between the border and value of the object
    InOut {
        /// inner top side padding
        inner_top: u16,
        /// inner bottom side padding
        inner_bottom: u16,
        /// inner right side padding
        inner_right: u16,
        /// inner lef tside padding
        inner_left: u16,
        /// outer top side padding
        outer_top: u16,
        /// outer bottom side padding
        outer_bottom: u16,
        /// outer right side padding
        outer_right: u16,
        /// outer left side padding
        outer_left: u16,
    },
}

impl Term {
    /// rescales the Term object dimensions to fit the new window size
    pub fn rescale(&mut self, wdiff: u16, hdiff: u16) {
        self.w *= wdiff;
        self.h *= hdiff;
        self.cx *= wdiff;
        self.cy *= hdiff;
    }
}

impl Container {
    /// rescales the Container object dimensions to fit the new window size
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
    /// rescales the Text object dimensions to fit the new window size
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
