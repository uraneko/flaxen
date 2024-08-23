// can have input, non editable or both
// so what are input or non editable?
// they are traits.
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

impl<T: Copy> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
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
enum Text<'a, const CLASS: char> {
    Input(Input<'a, CLASS>),
    NonEditable(NonEditable<'a, CLASS>),
    None,
}

impl<'a, const CLASS: char> Default for Text<'a, CLASS> {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug)]
pub struct Input<'a, const CLASS: char> {
    id: ID<'a>,
    value: Vec<char>,
    space: Space<usize>,
}

#[derive(Debug)]
pub struct NonEditable<'a, const CLASS: char> {
    id: ID<'a>,
    value: Vec<char>,
    space: Space<usize>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Space<T>
where
    T: Copy + Into<usize> + From<usize>,
{
    cursor: Point<T>,
    origin: Point<T>,
    width: T,
    height: T,
}

impl<T: Copy + Into<usize> + From<usize> + Div<Output = T>> Div<Space<T>> for Space<T> {
    type Output = Self;

    fn div(self, rhs: Space<T>) -> Self::Output {
        Self {
            cursor: self.cursor / rhs.cursor,
            origin: self.origin / rhs.origin,
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl<T: Copy + Into<usize> + From<usize> + Mul<Output = T>> Mul<Space<T>> for Space<T> {
    type Output = Self;

    fn mul(self, rhs: Space<T>) -> Self::Output {
        Self {
            cursor: self.cursor * rhs.cursor,
            origin: self.origin * rhs.origin,
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}

impl<T: Copy + Into<usize> + From<usize> + Add<Output = T>> Add<Space<T>> for Space<T> {
    type Output = Self;

    fn add(self, rhs: Space<T>) -> Self::Output {
        Self {
            cursor: self.cursor + rhs.cursor,
            origin: self.origin + rhs.origin,
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T: Copy + Into<usize> + From<usize> + Sub<Output = T>> Sub<Space<T>> for Space<T> {
    type Output = Self;

    fn sub(self, rhs: Space<T>) -> Self::Output {
        Self {
            cursor: self.cursor - rhs.cursor,
            origin: self.origin - rhs.origin,
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl Default for Space<usize> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cursor: Point::new(0, 0),
            origin: Point::new(0, 0),
        }
    }
}

impl<T: Copy> Space<T>
where
    T: Copy + Into<usize> + From<usize>,
{
    fn new(w: T, h: T, origin: Point<T>) -> Self {
        Self {
            width: w,
            height: h,
            origin,
            cursor: Point::<T>::new(0.into(), 0.into()),
        }
    }
}

pub type ID<'a> = &'a str;

trait Id {
    fn kind(&self) -> IDKind;
}

impl<'a> Id for ID<'a> {
    fn kind(&self) -> IDKind {
        match self[2..].split(|c: char| c.is_ascii()).count() {
            0 => IDKind::BufferImage,
            1 => IDKind::Component,
            2 => IDKind::TextInput,
            3 => IDKind::TextNE,
            4 => IDKind::Events,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum IDError {
    ProgramIsUnique,
}

#[derive(Debug)]
pub enum IDKind {
    BufferImage,
    Component,
    TextInput,
    // NonEditable Text
    TextNE,
    Events,
}

use std::io::StdoutLock;

use crate::Term;

struct InnerLogic;

impl<'a, const CLASS: char> Input<'a, CLASS> {}

type TextId = u8;

#[derive(Debug)]
pub struct Component<'a, 'b, const CLASS: char>
where
    'a: 'b,
{
    id: ID<'b>,
    items: [Text<'a, CLASS>; 5],
    space: Space<usize>,
}

impl<'a, 'b, const CLASS: char> std::fmt::Display for Component<'a, 'b, CLASS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

// [IMPORTANT]NOTE:
// the Commissioner handles all ID matters
// he also handles all Events matters
// and all Space allocation matters
// he is the only one with access to the LayerMap

impl<'a, 'b, const CLASS: char> Component<'a, 'b, CLASS> {
    pub fn new(id: ID<'b>, space: Space<usize>) -> Self {
        Self {
            id,
            items: Default::default(),
            space,
        }
    }

    fn resize() {}
}

#[cfg(test)]
mod tests {

    fn test_const_struct_impl_trait() {
        let a = A::<'a'>::new(8);
        let b = A::<'b'>::new(9);

        <A<'a'> as G<InnerLogic>>::a(&a); // doesnt error

        // <A<'b'> as G<InnerLogic>>::a(&b); // errors

        let c = A::<'a'>::new(34);

        <A<'a'> as G<InnerLogic>>::a(&c); // doesnt error
    }

    struct InnerLogic;

    struct A<const ID: char> {
        id: u8,
    }

    impl<const ID: char> A<ID> {
        const fn id(&self) -> u8 {
            self.id
        }

        fn new(id: u8) -> Self {
            Self { id }
        }
    }

    trait G<T> {
        const IDMatch: char;

        fn a(&self);
    }

    impl<InnerLogic> G<InnerLogic> for A<'a'> {
        const IDMatch: char = 'a';

        fn a(&self) {
            println!("from G<InnerLogic>: a(): {}", self.id);
        }
    }
}
