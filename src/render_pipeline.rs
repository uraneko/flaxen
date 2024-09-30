use std::collections::HashMap;
use std::io::{StdoutLock, Write};
use std::str::Chars;

use crate::components::*;
use crate::space::{border::Border, padding::Padding};
use crate::themes::Style;

pub mod container;
pub mod term;
pub mod text;

// NOTE: an object can not be initialized unless
// its id is valid,
// its dimensions are valid, including overlay

// NOTE: the render methods here depend on the value field of text objects having a text.w * text.h  len

// TODO: should process only if values change
// otherwise just render

pub(crate) fn spread_padding(p: &Padding) -> [u16; 8] {
    match p {
        Padding::None => [0; 8],
        Padding::Inner {
            top,
            bottom,
            right,
            left,
        } => [0, 0, 0, 0, *right, *left, *top, *bottom],
        Padding::Outer {
            top,
            bottom,
            right,
            left,
        } => [*right, *left, *top, *bottom, 0, 0, 0, 0],
        Padding::InOut {
            inner_top,
            inner_bottom,
            inner_right,
            inner_left,
            outer_top,
            outer_bottom,
            outer_right,
            outer_left,
        } => [
            *outer_right,
            *outer_left,
            *outer_top,
            *outer_bottom,
            *inner_right,
            *inner_left,
            *inner_top,
            *inner_bottom,
        ],
    }
}

fn log_buf(buf: &[Option<char>], w: u16, h: u16) {
    print!("lines");
    for ih in 0..h {
        println!("");
        for iw in 0..w {
            print!("{}", (buf[(iw + ih * w) as usize]).unwrap_or(' '));
        }
    }
    println!("");
}
