use std::collections::HashMap;
use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, RemAssign, ShlAssign, ShrAssign, Sub,
        SubAssign,
    },
};

use crate::components::{ComponentTree, Container, Term, Text};
use crate::render_pipeline;

pub mod border;
pub mod padding;

use border::Border;
use padding::Padding;

pub(crate) fn between<T: std::cmp::PartialOrd>(a: T, b: T, c: T) -> bool {
    a < b && b < c
}

// this exists because the border doesn't have a width and height in this lib, it is always a
// single cell box surrounding the value
// NOTE: this need be used only for Border::Manual
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

// calculates the absolute origin of a text object in terminal display coordinates
pub(crate) fn calc_text_abs_ori(
    id: &[u8; 2],
    ori: &[u16; 2],
    ib: &Border,
    ip: &Padding,
    cont: &Container,
) -> [u16; 2] {
    let [ix0, iy0] = ori;
    let [_, cpol, cpot, _, _, cpil, cpit, _] = render_pipeline::spread_padding(&cont.padding);
    let cb = if let Border::None = cont.border { 0 } else { 1 };

    let [_, ipol, ipot, _, _, ipil, ipit, _] = render_pipeline::spread_padding(&ip);
    let ib = if let Border::None = ib { 0 } else { 1 };

    [
        cpol + cb + cpil + cont.x0 + ipol + ib + ipil + ix0 + 1,
        cpot + cb + cpit + cont.y0 + ipot + ib + ipit + iy0,
    ]
}

pub(crate) fn resolve_wh(border: &Border, padding: &Padding) -> [u16; 2] {
    let bv = if let Border::None = border { 0 } else { 1 };
    let [pw, ph] = if let Padding::Inner {
        top,
        bottom,
        right,
        left,
    } = padding
    {
        [right + left, top + bottom]
    } else if let Padding::Outer {
        top,
        bottom,
        right,
        left,
    } = padding
    {
        [right + left, top + bottom]
    } else if let Padding::InOut {
        inner_top,
        inner_bottom,
        inner_right,
        inner_left,
        outer_top,
        outer_bottom,
        outer_right,
        outer_left,
    } = padding
    {
        [
            inner_right + inner_left + outer_right + outer_left,
            inner_top + inner_bottom + outer_top + outer_bottom,
        ]
    } else {
        [0; 2]
    };

    [pw + bv * 2, ph + bv * 2]
}

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

// use this instead of width/height
#[derive(Debug, Clone)]
pub enum Polygon {
    Square {
        top_left: u16,
        top_right: u16,
        bottom_left: u16,
        bottom_right: u16,
    },

    Rectangle {
        top_left: u16,
        top_right: u16,
        bottom_left: u16,
        bottom_right: u16,
    },

    Triangle {
        angle: u16,
        e1: u16,
        e2: u16,
        e3: u16,
    },

    Line {
        angle: u16,
        e1: u16,
        e2: u16,
    },

    Free {
        vertices: Vec<u16>,
    },
}

impl Polygon {
    fn square(top_left: u16, top_right: u16, bottom_right: u16, bottom_left: u16) -> Self {
        Self::Square {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    fn rectangle(top_left: u16, top_right: u16, bottom_right: u16, bottom_left: u16) -> Self {
        Self::Rectangle {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    fn triangle(e1: u16, e2: u16, e3: u16, angle: u16) -> Self {
        Self::Triangle { angle, e1, e2, e3 }
    }

    fn line(angle: u16, e1: u16, e2: u16) -> Self {
        Self::Line { angle, e1, e2 }
    }

    fn free(vertices: Vec<u16>) -> Self {
        Self::Free { vertices }
    }
}

// can be either vertical or horizontal
// use instead of passing x0 and y0
#[derive(Debug, Clone, Default)]
pub enum Pos {
    /// position component at the start of parent's area, either vertically or horizontally
    Start,
    /// center component inisde parent's area, either vertically or horizontally
    #[default]
    Center,
    /// position component at the end of parent's area, either vertically or horizontally
    End,
    /// custom component position, can be vertical or horizontal
    Value(u16),
}

impl Pos {
    // can be used for wither horizontal or vertical coordinate
    pub(super) fn position(self, value: u16) -> u16 {
        match self {
            Self::Start => 0,
            Self::End => value,
            Self::Center => value / 2,
            Self::Value(value) => value,
        }
    }

    // self is horizontal x0
    // other is vertical coordinate y0
    pub(super) fn point(self, other: Self, values: [u16; 2]) -> [u16; 2] {
        [self.position(values[0]), other.position(values[1])]
    }
}

#[derive(Debug, Clone, Default)]
pub enum Area {
    #[default]
    Zero,
    Fill,
    Values {
        w: u16,
        h: u16,
    },
}

impl Area {
    pub fn width(&self) -> Option<u16> {
        if let Self::Values { w, .. } = self {
            return Some(*w);
        }

        None
    }

    pub fn height(&self) -> Option<u16> {
        if let Self::Values { h, .. } = self {
            return Some(*h);
        }

        None
    }

    pub fn unwrap(self, values: [u16; 2]) -> [u16; 2] {
        match self {
            // FIXME: if area is fill, need to pass on the parent and
            // substract the children areas
            Self::Fill => values,
            Self::Zero => [0; 2],
            Self::Values { w, h } => [w, h],
        }
    }
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
