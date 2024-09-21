use crate::components::{ComponentTree, Container, Term, Text};
use crate::render_pipeline;

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

// TODO!
pub enum DisplayLayout {
    /// hidden component
    None,
    /// no additional layout rules are applied on the children
    Canvas,
    /// children are displayed in a flex style
    /// for more customization add a "flex" map property to this component
    /// with the needed properties
    Flex,
    /// children are displayed in a grid style
    /// for more customization add a "grid" map property to this component
    /// with the needed properties
    Grid,
}

// can be either vertical or horizontal
// use instead of passing x0 and y0
#[derive(Debug, Clone)]
pub enum Pos {
    /// position component at the start of parent's area, either vertically or horizontally
    Start,
    /// center component inisde parent's area, either vertically or horizontally
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

pub enum Area {
    Zero,
    Fill,
    Values { w: u16, h: u16 },
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

// FIXME: since the Manual variant takes 'static lifetimed strs
// it can not be created nor its method manual() used with strs that are static like those gotten
// from String::as_str
// keep like this or make it take String
// TODO: find out why i put copy trait on Border enum
impl Border {
    /// creates a new Border with the None variant
    pub fn none() -> Self {
        Self::None
    }

    /// creates a new Border with the Uniform variant
    pub fn uniform(value: char) -> Self {
        Self::Uniform(value)
    }

    /// creates a new Border with the Polyform variant
    pub fn polyform(
        tlcorner: char,
        trcorner: char,
        brcorner: char,
        blcorner: char,
        rl: char,
        tb: char,
    ) -> Self {
        Self::Polyform {
            trcorner,
            tlcorner,
            brcorner,
            blcorner,
            rl,
            tb,
        }
    }

    /// creates a new Border with the Manual variant
    pub fn manual(
        tlcorner: char,
        trcorner: char,
        brcorner: char,
        blcorner: char,
        t0: &'static str,
        tp: char,
        t1: &'static str,
        r0: &'static str,
        rp: char,
        r1: &'static str,
        l0: &'static str,
        lp: char,
        l1: &'static str,
        b0: &'static str,
        bp: char,
        b1: &'static str,
    ) -> Self {
        Self::Manual {
            trcorner,
            tlcorner,
            brcorner,
            blcorner,
            t0,
            tp,
            t1,
            r0,
            rp,
            r1,
            l0,
            lp,
            l1,
            b0,
            bp,
            b1,
        }
    }

    pub fn mono(self, mono: char) -> Self {
        if let Self::Uniform(ch) = self {
            return Self::Uniform(mono);
        }

        self
    }

    pub fn trcorner(self, trcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            blcorner,
            brcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            blcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn tlcorner(self, tlcorner: char) -> Self {
        if let Self::Polyform {
            trcorner,
            blcorner,
            brcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            blcorner,
            trcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn brcorner(self, brcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            blcorner,
            trcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            trcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn blcorner(self, blcorner: char) -> Self {
        if let Self::Polyform {
            tlcorner,
            brcorner,
            trcorner,
            rl,
            tb,
            ..
        } = self
        {
            return Self::Polyform {
                trcorner,
                tlcorner,
                blcorner,
                brcorner,
                rl,
                tb,
            };
        } else if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn t0(self, t0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn t1(self, t1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            tp,
            t0,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn b0(self, b0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn b1(self, b1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn r0(self, r0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn r1(self, r1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn l0(self, l0: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn l1(self, l1: &'static str) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,
            r0,
            rp,
            r1,

            l0,
            lp,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn tp(self, tp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn bp(self, bp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            lp,
            l1,

            b0,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn rp(self, rp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            r1,

            l0,
            lp,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }

    pub fn lp(self, lp: char) -> Self {
        if let Self::Manual {
            tlcorner,
            trcorner,
            brcorner,
            blcorner,

            t0,
            tp,
            t1,

            r0,
            rp,
            r1,

            l0,
            l1,

            b0,
            bp,
            b1,
            ..
        } = self
        {
            return Self::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,

                t0,
                tp,
                t1,

                r0,
                rp,
                r1,

                l0,
                lp,
                l1,

                b0,
                bp,
                b1,
            };
        }

        self
    }
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

impl Padding {
    /// creates a new Padding with the None variant
    pub fn none() -> Self {
        Padding::None
    }

    /// creates a new Padding with the Inner variant
    pub fn inner(top: u16, bottom: u16, right: u16, left: u16) -> Self {
        Self::Inner {
            top,
            bottom,
            right,
            left,
        }
    }

    /// creates a new Padding with the Outer variant
    pub fn outer(top: u16, bottom: u16, right: u16, left: u16) -> Self {
        Self::Inner {
            top,
            bottom,
            right,
            left,
        }
    }

    pub fn in_out(
        inner_top: u16,
        inner_bottom: u16,
        inner_right: u16,
        inner_left: u16,
        outer_top: u16,
        outer_bottom: u16,
        outer_right: u16,
        outer_left: u16,
    ) -> Self {
        Self::InOut {
            inner_top,
            inner_bottom,
            inner_right,
            inner_left,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
        }
    }

    /// mutates the top padding value and returns the padding enum value
    /// or returns self value in case variant is neither Inner nor Outer
    pub fn top(self, top: u16) -> Self {
        if let Self::Inner {
            bottom,
            right,
            left,
            ..
        } = self
        {
            return Self::Inner {
                bottom,
                right,
                left,
                top,
            };
        } else if let Self::Outer {
            bottom,
            right,
            left,
            ..
        } = self
        {
            return Self::Outer {
                bottom,
                right,
                left,
                top,
            };
        }

        self
    }

    /// mutates the buttom padding value and returns the padding enum value
    /// or returns self value in case variant is neither Inner nor Outer
    pub fn bottom(self, bottom: u16) -> Self {
        if let Self::Inner {
            top, right, left, ..
        } = self
        {
            return Self::Inner {
                bottom,
                right,
                left,
                top,
            };
        } else if let Self::Outer {
            top, right, left, ..
        } = self
        {
            return Self::Outer {
                bottom,
                right,
                left,
                top,
            };
        }

        self
    }

    /// mutates the right padding value and returns the padding enum value
    /// or returns self value in case variant is neither Inner nor Outer
    pub fn right(self, right: u16) -> Self {
        if let Self::Inner {
            top, bottom, left, ..
        } = self
        {
            return Self::Inner {
                bottom,
                right,
                left,
                top,
            };
        } else if let Self::Outer {
            top, bottom, left, ..
        } = self
        {
            return Self::Outer {
                bottom,
                right,
                left,
                top,
            };
        }

        self
    }

    /// mutates the left padding value and returns the padding enum value
    /// or returns self value in case variant is neither Inner nor Outer
    pub fn left(self, left: u16) -> Self {
        if let Self::Inner {
            top, bottom, right, ..
        } = self
        {
            return Self::Inner {
                bottom,
                right,
                left,
                top,
            };
        } else if let Self::Outer {
            top, bottom, right, ..
        } = self
        {
            return Self::Outer {
                bottom,
                right,
                left,
                top,
            };
        }

        self
    }

    /// mutates the inner_top padding value and returns the padding enum value
    /// or returns self value in case variant is not InOut
    pub fn inner_top(self, inner_top: u16) -> Self {
        if let Self::InOut {
            inner_bottom,
            inner_left,
            inner_right,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
            ..
        } = self
        {
            return Self::InOut {
                inner_top,
                inner_bottom,
                inner_left,
                inner_right,
                outer_top,
                outer_bottom,
                outer_right,
                outer_left,
            };
        }

        self
    }

    /// mutates the inner_bottom padding value and returns the padding enum value
    /// or returns self value in case variant is not InOut
    pub fn inner_bottom(self, inner_bottom: u16) -> Self {
        if let Self::InOut {
            inner_top,
            inner_left,
            inner_right,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
            ..
        } = self
        {
            return Self::InOut {
                inner_top,
                inner_bottom,
                inner_left,
                inner_right,
                outer_top,
                outer_bottom,
                outer_right,
                outer_left,
            };
        }

        self
    }

    /// mutates the inner_right padding value and returns the padding enum value
    /// or returns self value in case variant is not InOut
    pub fn inner_right(self, inner_right: u16) -> Self {
        if let Self::InOut {
            inner_top,
            inner_left,
            inner_bottom,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
            ..
        } = self
        {
            return Self::InOut {
                inner_top,
                inner_bottom,
                inner_left,
                inner_right,
                outer_top,
                outer_bottom,
                outer_right,
                outer_left,
            };
        }

        self
    }

    /// mutates the inner_left padding value and returns the padding enum value
    /// or returns self value in case variant is not InOut
    pub fn inner_left(self, inner_left: u16) -> Self {
        if let Self::InOut {
            inner_top,
            inner_right,
            inner_bottom,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
            ..
        } = self
        {
            return Self::InOut {
                inner_top,
                inner_bottom,
                inner_left,
                inner_right,
                outer_top,
                outer_bottom,
                outer_right,
                outer_left,
            };
        }

        self
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
