use crate::object_tree::{Container, ObjectTree, Term, Text, Zero};

use std::collections::HashMap;
use std::{
    fmt::{Debug, Display},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, RemAssign, ShlAssign, ShrAssign, Sub,
        SubAssign,
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<T>
where
    T: Copy,
{
    x: T,
    y: T,
}

impl Default for Point<usize> {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl<T: Copy + Div<Output = T>> Div<Point<T>> for Point<T> {
    type Output = Self;

    fn div(self, rhs: Point<T>) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<Point<T>> for Point<T> {
    type Output = Self;

    fn mul(self, rhs: Point<T>) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T: Copy + Add<Output = T>> Add<Point<T>> for Point<T> {
    type Output = Self;

    fn add(self, rhs: Point<T>) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Copy + Sub<Output = T>> Sub<Point<T>> for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Point<T>) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + MulAssign> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn re(&mut self, x: T, y: T) {
        self.x = x;
        self.y = y;
    }

    pub fn re_x(&mut self, x: T) -> T {
        let dis_x = self.x;
        self.x = x;

        dis_x
    }

    pub fn re_y(&mut self, y: T) -> T {
        let dis_y = self.y;
        self.y = y;

        dis_y
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    pub fn rescale(&mut self, wd: T, hd: T) {
        self.x *= wd;
        self.y *= hd;
    }

    pub fn place(&mut self, x: T, y: T) {
        self.x = x;
        self.y = y;
    }
}

impl<T: Copy + DivAssign + Div<Output = T>> DivAssign for Point<T> {
    fn div_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T: Copy + MulAssign + Mul<Output = T>> MulAssign for Point<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T: Copy + SubAssign + Sub<Output = T>> SubAssign for Point<T> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Copy + AddAssign + Add<Output = T>> AddAssign for Point<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<
        T: Copy
            + Sub<Output = T>
            + Add<Output = T>
            + Mul<Output = T>
            + Div<Output = T>
            + SubAssign
            + AddAssign,
    > Point<T>
{
    fn shrink(&mut self, x: T, y: T) {
        self.x -= x;
        self.y -= y;
    }

    fn grow(&mut self, x: T, y: T) {
        self.x += x;
        self.y += y;
    }
}

#[derive(Debug)]
pub enum Border {
    None,
    Some(char),
}

#[derive(Debug)]
pub enum Padding {
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

impl SpaceAwareness for Term {
    fn rescale(&mut self, wdiff: u16, hdiff: u16) {
        self.width *= wdiff;
        self.height *= hdiff;
        self.cursor.rescale(wdiff, hdiff);
        // TODO: self buf changes
    }
}

impl SpaceAwareness for Container {
    fn rescale(&mut self, wdiff: u16, hdiff: u16) {
        self.width *= wdiff;
        self.height *= hdiff;
        self.origin.rescale(wdiff, hdiff);
    }
}

impl SpaceAwareness for Text {
    fn rescale(&mut self, wdiff: u16, hdiff: u16) {
        self.width *= wdiff;
        self.height *= hdiff;
        self.origin.rescale(wdiff, hdiff);
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
    // responds to window resize events
    fn rescale(&mut self, wdiff: u16, hdiff: u16);
}
