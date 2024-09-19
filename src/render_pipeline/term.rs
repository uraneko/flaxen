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

        for bidx in 0..cells.len() {
            match cells[bidx] {
                Some(c) => {
                    s.clear();
                    s.push(c);
                    _ = writer.write(&s.as_bytes());
                }
                None => {
                    _ = writer.write(b"\x1b[C");
                }
            };

            // increment index after every cell write or movement
            idx += 1;
            // if we have reached end of line
            if idx == self.w {
                // we increment current line by one
                line += 1;
                // we reset idx
                idx = 0;

                // we move the cursor to the next line's first cell
                writer.write(&[13, 10]);
            }
        }

        assert_eq!(line, self.h);

        let pos = format!("\x1b[{};{}f", self.cy, self.cx);
        _ = writer.write(pos.as_bytes());
    }

    /// clears the whole terminal display
    pub fn clear(&self, writer: &mut StdoutLock) {
        writer.write(b"\x1b[H\x1b[J");
    }

    /// places the cursor at the new position
    pub fn place(&mut self, x: u16, y: u16) {
        let esc_seq = format!("\x1b{};{}f", x, y);
        self.cx = x;
        self.cy = y;
    }
}
