use std::collections::HashMap;
use std::io::{StdoutLock, Write};
use std::str::Chars;

use crate::components::*;
use crate::space::{Border, Padding};
use crate::themes::Style;

use super::spread_padding;

impl Term {
    /// renders the cursor in the self cx, cy position
    pub fn render_cursor(&self, writer: &mut StdoutLock) {
        let pos = format!("\x1b[{};{}f", self.cy, self.cx);
        _ = writer.write(pos.as_bytes());
    }

    /// renders only the text objects that have seen some value/border change since the last event
    /// loop iteration, either through user interaction or some background events being triggered
    pub fn live_render(&self, writer: &mut StdoutLock) {
        self.changed().iter().for_each(|t| match t.change {
            2 => t.render_value(writer),
            4 => t.render_border(writer),
            6 => t.render(writer),
            _ => unreachable!(""),
        });
    }

    /// renders only the demanded components' parts,
    /// decides what to render based on some property fields's key and values for border, value and
    /// all rendeing
    /// if a child component has the key in its properties and the key value matches at least one
    /// of br (border render), vr (value render) or ar (all render) then the matching part gets
    /// rendered
    /// implementing this deprecates live_render
    pub fn partial_render(
        &self,
        writer: &mut StdoutLock,
        key: &str,
        br: Property,
        vr: Property,
        ar: Property,
    ) {
    }

    fn prepare(&self) -> (Vec<Option<char>>) {
        let mut lines: Vec<Option<char>> = vec![];
        lines.resize((self.w * self.h) as usize, None);

        self.containers.iter().for_each(|c| {
            let mut idx = c.x0 + c.y0 * self.w;
            let mut line = 0;
            let (cells, [cwx, chx]) = c.prepare();

            loop {
                // write the item line inside the container lines
                for cidx in 0..cwx {
                    let cell = cells[(cidx + line * cwx) as usize];
                    if cell.is_some() {
                        lines[idx as usize] = cell;
                    }
                    idx += 1;
                }

                // skip the end of current line that is not inside item twx
                // then skip the next line beginning up to c.xy
                idx += self.w - cwx;

                // increment item lines by one until last line
                line += 1;
                if line == chx {
                    break;
                }
            }
        });

        // NOTE: this part is really hard to debug since term is the size of the entire terminal
        // window and has no border or padding
        // but all the parts before this are working (sans the already found bugs)
        // and this part too seems to be working
        lines
    }

    /// renders the whole buffer into the terminal
    /// assumes that Term.clear() has been used before hand to prepare the terminal display for the
    /// rendering
    // FIXME: when this is used themes are not applied, contrary to individual object render methods
    // this is expected behavior, although it's bad
    // need a way to map whatever style to some range of positions in the term buffer
    // that way, atomic style implementation becomes easy to call from anywhere
    pub fn render(&mut self, writer: &mut StdoutLock) {
        let cells = self.prepare();

        let mut s = String::new();

        let mut line = 0;
        let mut idx = 0;

        cells.iter().for_each(|c| {
            if let Some(ch) = c {
                // print!("found char, ");
                s.push(*ch);
            } else {
                // print!("found space, ");
                s.push_str("\x1b[C");
            }
            idx += 1;
            if idx == self.w {
                idx = 0;
                line += 1;
                if line < self.h - 1 {
                    // println!("breaking line at {{{}}}", &s[s.len() - 1..s.len()]);
                    s.push_str("\r\n");
                }
            }
        });

        assert_eq!(line, self.h);

        let pos = format!("\x1b[{};{}f", self.cy, self.cx);
        s.push_str(&pos);
        // println!("{}", s);
        _ = writer.write(s.as_bytes());
        _ = writer.flush();
    }

    /// clears the whole terminal display
    /// first implementation of clear
    pub fn clear(&self, writer: &mut StdoutLock) {
        writer.write(b"\x1b[H\x1b[J");
    }

    /// clears the whole terminal display
    /// second implementation of clear
    pub fn clear1(&self, writer: &mut StdoutLock) {
        let mut s = String::from("\x1b[H");
        (0..self.h)
            .into_iter()
            .for_each(|_| s.push_str("\x1b[2K\x1b[C"));
        s.push_str("\x1b[H");
        writer.write(s.as_bytes());
    }
}
