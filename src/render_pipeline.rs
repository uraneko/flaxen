use crate::object_tree::*;
use crate::space::{Border, Padding};
use crate::themes::Style;

use std::collections::HashMap;
use std::io::{StdoutLock, Write};

// NOTE: an object can not be initialized unless
// its id is valid,
// its dimensions are valid, including overlay

impl Term {
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

impl Container {
    // renders container border and children
    pub fn render(&self, writer: &mut StdoutLock) {
        self.render_border(writer);
        self.render_value(writer);
    }

    // renders only the items inside the container without rendering their borders
    pub fn render_value(&self, writer: &mut StdoutLock) {
        let [_, pol, pot, _, _, pil, pit, _] = spread_padding(&self.padding);
        let cb = if let Border::None = self.border { 0 } else { 1 };

        self.items.iter().for_each(|t| {
            let [_, tpol, tpot, _, _, tpil, tpit, _] = spread_padding(&t.padding);
            let tb = if let Border::None = t.border { 0 } else { 1 };

            let ori = [
                self.x0 + pol + cb + pil + t.x0 + tpol + tb + tpil + 1,
                self.y0 + pot + cb + pit + t.y0 + tpot + tb + tpit,
            ];

            t.render_value(writer);
        });
    }

    // renders only the container border
    pub fn render_border(&self, writer: &mut StdoutLock) {
        let [_, pol, pot, _, pir, pil, pit, pib] = spread_padding(&self.padding);
        let [xb, yb] = [self.x0 + pol + 1, self.y0 + pot];
        let mut s = format!("{}\x1b[{};{}f", &self.bstyle, yb, xb);

        let wb = pil + 1 + self.w + 1 + pir;
        let hb = pit + 1 + self.h + 1 + pib;

        if let Border::Uniform(c) = self.border {
            for _ in 0..wb {
                s.push(c)
            }

            for idx in 1..hb - 1 {
                s.push_str(&format!(
                    "\x1b[{};{}f{}\x1b[{};{}f{}",
                    yb + idx,
                    xb,
                    c,
                    yb + idx,
                    xb + wb - 1,
                    c
                ));
            }

            s.push_str(&format!("\x1b[{};{}f", yb + hb - 1, xb));
            for _ in 0..wb {
                s.push(c)
            }

            s.push_str("\x1b[0m");

            writer.write(s.as_bytes());
        }
    }

    // adds padding and border to the width and height of the container
    // should be called from the sef render method
    pub fn decorate(&self) -> [u16; 2] {
        let [mut wextra, mut hextra] = match self.border {
            Border::None => [self.w, self.h],
            _ => [self.w + 2, self.h + 2],
        };

        [wextra, hextra] = match self.padding {
            Padding::None => [wextra, hextra],
            Padding::Inner {
                top,
                bottom,
                right,
                left,
            } => [wextra + right + left, hextra + top + bottom],
            Padding::Outer {
                top,
                bottom,
                right,
                left,
            } => [wextra + right + left, hextra + top + bottom],
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
                wextra + inner_right + inner_left + outer_right + outer_left,
                hextra + inner_top + inner_bottom + outer_top + outer_bottom,
            ],
        };

        [wextra, hextra]
    }

    // prepares the border and paddings of the container
    // then calls all the self items prepare methods
    pub fn prepare(&self) -> (Vec<Option<char>>, [u16; 2]) {
        // make out each line of the item, padding and border included
        // then render line
        // until all lines are rendered
        let [por, pol, pot, pob, pir, pil, pit, pib] = spread_padding(&self.padding);
        let brdr = match self.border {
            Border::None => 0,
            _ => 1,
        };

        let mut lines: Vec<Option<char>> = vec![];

        // wx is the number of chars in a line
        // hx is the number of lines
        let [wx, mut hx] = self.decorate();
        lines.resize((wx * hx) as usize, None);

        self.process(&mut lines);

        self.items.iter().for_each(|t| {
            let mut idx = pol + brdr + pil + t.x0 + (pot + brdr + pit + t.y0) * wx;
            let mut line = 0;
            let (cells, [twx, thx]) = t.prepare();

            loop {
                // write the item line inside the container lines
                for tidx in 0..twx {
                    let cell = cells[(tidx + line * twx) as usize];
                    if cell.is_some() {
                        lines[idx as usize] = cell;
                    }
                    idx += 1;
                }

                // skip the end of current line that is not inside item twx
                // then skip the next line beginning up to t.xy
                idx += wx - twx;

                // increment item lines by one until last line
                line += 1;
                if line == thx {
                    break;
                }
            }
        });

        // log_buf(&lines, wx, hx);
        (lines, [wx, hx])
    }

    fn process(&self, lines: &mut Vec<Option<char>>) {
        let [por, pol, pot, pob, pir, pil, pit, pib] = spread_padding(&self.padding);

        let [wx, hx] = self.decorate();

        // we skip as many lines as the value of padding outer top
        // if padding outer bottom > 0 then lst line gets nothing
        match self.border {
            Border::None => return,

            Border::Uniform(c) => {
                self.process_uniform(c, lines, wx, hx, por, pol, pot, pob, pir, pil, pit, pib)
            }

            Border::Polyform {
                rcorner,
                lcorner,
                tcorner,
                bcorner,
                rl,
                tb,
            } => {}
        }
    }

    fn process_none(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
    }

    fn process_uniform(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
        {
            // first line of border
            // lines of padding times number of cells in one line
            let mut idx = pot * wx;
            let mut line = pot;
            // we skip the outer left padding values
            idx += pol;
            // we fill value length + inner padding right + left with border value
            for i in 0..pil + 1 + self.w + pir + 1 {
                lines[idx as usize] = Some(c);
                idx += 1;
            }
            // we skipp the outer right padding
            idx += por;
            // first bordered line ends
            line += 1;

            // handle the pre value lines
            // every iteration is a line

            while line < pot + 1 + pit + self.h + pib {
                // new line we skip padding outer left
                idx += pol;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip inner left and right padding and the value width
                idx += pil + self.w + pir;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skipp outer right padding
                idx += por;

                line += 1;
                // we are in front of the second full border line
            }

            // second and last line of full border
            // we skip the outer left padding values
            idx += pol;
            // we fill value length + inner padding right + left with border value
            for i in 0..pil + 1 + self.w + pir + 1 {
                lines[idx as usize] = Some(c);
                idx += 1;
            }
            // we skip the outer right padding
            idx += por;
            // second bordered line ends
            line += 1;
            assert_eq!(line + pob, pit + pot + self.h + pib + pob + 2);
        }
    }

    fn process_polyform(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
    }
}

impl Text {
    // renders both the text border and value
    pub fn render(&self, writer: &mut StdoutLock) {
        self.render_border(writer);
        self.render_value(writer);
    }

    // renders only the text border
    pub fn render_border(&self, writer: &mut StdoutLock) {
        let [por, pol, pot, pob, pir, pil, pit, pib] = spread_padding(&self.padding);
        let [xb, yb] = [self.ax0 - pil - 1, self.ay0 - pit - 1];
        let mut s = format!("{}\x1b[{};{}f", &self.bstyle, yb, xb);

        let wb = pil + 1 + self.w + 1 + pir;
        let hb = pit + 1 + self.h + 1 + pib;

        if let Border::Uniform(c) = self.border {
            for _ in 0..wb {
                s.push(c)
            }

            for idx in 1..hb - 1 {
                s.push_str(&format!(
                    "\x1b[{};{}f{}\x1b[{};{}f{}",
                    yb + idx,
                    xb,
                    c,
                    yb + idx,
                    xb + wb - 1,
                    c
                ));
            }

            s.push_str(&format!("\x1b[{};{}f", yb + hb - 1, xb));
            for _ in 0..wb {
                s.push(c)
            }

            s.push_str("\x1b[0m");

            writer.write(s.as_bytes());
        }
    }

    // renders only the text value
    pub fn render_value(&self, writer: &mut StdoutLock) {
        let h0 = self.ay0;

        let del = |s: &mut String, y: u16| {
            *s += &format!("\x1b[{};{}f\x1b[{}X", y, self.ax0, self.w);
        };

        let put = |s: &mut String, y: u16| {
            *s += &format!("\x1b[{};{}f", h0 + y, self.ax0);
            for idx in 0..self.w {
                let c = self.value[(idx + y * self.w) as usize];
                if c.is_some() {
                    s.push(c.unwrap());
                } else {
                    s.push_str("\x1b[C")
                };
            }
        };

        let mut s = format!("{}", &self.vstyle);

        // iterate through lines
        for idx in 0..self.h {
            del(&mut s, h0 + idx);
            put(&mut s, idx);
        }

        s += "\x1b[0m";

        writer.write(s.as_bytes());
    }

    pub fn decorate(&self) -> [u16; 2] {
        let [mut wextra, mut hextra] = match self.border {
            Border::None => [self.w, self.h],
            _ => [self.w + 2, self.h + 2],
        };

        [wextra, hextra] = match self.padding {
            Padding::None => [wextra, hextra],
            Padding::Inner {
                top,
                bottom,
                right,
                left,
            } => [wextra + right + left, hextra + top + bottom],
            Padding::Outer {
                top,
                bottom,
                right,
                left,
            } => [wextra + right + left, hextra + top + bottom],
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
                wextra + inner_right + inner_left + outer_right + outer_left,
                hextra + inner_top + inner_bottom + outer_top + outer_bottom,
            ],
        };

        [wextra, hextra]
    }

    // this should be used inside the container prepare method
    pub fn prepare(&self) -> (Vec<Option<char>>, [u16; 2]) {
        // make out each line of the item, padding and border included
        // then render line
        // until all lines are rendered
        let mut lines: Vec<Option<char>> = vec![];
        // wx is the number of chars in a line
        // hx is the number of lines
        let [wx, mut hx] = self.decorate();

        lines.resize((wx * hx) as usize, None);

        self.process(&mut lines);

        (lines, [wx, hx])
    }

    fn process(&self, lines: &mut Vec<Option<char>>) {
        let [por, pol, pot, pob, pir, pil, pit, pib] = spread_padding(&self.padding);

        let [wx, hx] = self.decorate();

        // we skip as many lines as the value of padding outer top
        // if padding outer bottom > 0 then lst line gets nothing
        match self.border {
            Border::None => return,

            Border::Uniform(c) => {
                self.process_uniform(c, lines, wx, hx, por, pol, pot, pob, pir, pil, pit, pib)
            }

            Border::Polyform {
                rcorner,
                lcorner,
                tcorner,
                bcorner,
                rl,
                tb,
            } => {}
        }
    }

    fn process_none(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
    }

    fn process_uniform(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
        {
            // the line the value starts on
            let v0 = pot + 1 + pit;
            // the line the value ends on
            let v1 = v0 + self.h;
            // println!("{}: v0 = {}, v1 = {}", line!(), v0, v1,);
            // println!("{}: wextra = {}, hxtra = {}", line!(), wx, hx);

            // first line of border
            // lines of padding times number of cells in one line
            let mut idx = pot * wx;
            let mut line = pot;
            // println!("{}: idx = {}, line = {}", line!(), idx, line,);
            // we skip the outer left padding values
            idx += pol;

            // log_buf(&lines, wx, hx);

            // we fill value length + inner padding right + left with border value
            for i in 0..pil + 1 + self.w + pir + 1 {
                lines[idx as usize] = Some(c);
                idx += 1;
            }
            // println!("lines ==> {:?}", lines);
            // log_buf(&lines, wx, hx);
            // we skipp the outer right padding
            idx += por;
            // first bordered line ends
            line += 1;
            // println!("{}: idx = {}, line = {}", line!(), idx, line,);

            // handle the pre value lines
            // every iteration is a line
            // we are still not in front of the value lines
            while line < v0 {
                // new line we skip padding outer left
                idx += pol;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip inner left and right padding and the value len
                idx += pil + self.w + pir;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip outer right padding
                idx += por;

                line += 1;
                // println!("{}: idx = {}, line = {}", line!(), idx, line,);
                // log_buf(&lines, wx, hx);
            }

            // handle the value lines
            // every iteration is a line
            // we are not out of the value lines yet
            while line < v1 {
                // println!("==>> line = {}", line);
                // new line we skip padding outer left
                idx += pol;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip inner left padding
                idx += pil;
                // write values
                for vi in 0..self.w as usize {
                    // println!(
                    //     "{}: vi{} + (w{} * (line{} - pot{} - 1 - pit{})) = {}",
                    //     line!(),
                    //     vi,
                    //     self.w,
                    //     line,
                    //     pot,
                    //     pit,
                    //     vi + (self.w * (line - pot - 1 - pit)) as usize
                    // );
                    let i = vi + (self.w * (line - pot - 1 - pit)) as usize;
                    if i < self.value.len() {
                        lines[idx as usize] = self.value[i];
                    }
                    idx += 1;
                    // log_buf(&lines, wx, hx);
                }
                // skip inner right padding
                idx += pir;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip outer right padding
                idx += por;

                line += 1;
                // println!("{}: idx = {}, line = {}", line!(), idx, line,);
                // log_buf(&lines, wx, hx);
            }
            // we left value lines

            // while we are not in front of the second full border line yet
            while line < pot + 1 + pit + self.h + pib {
                // new line we skip padding outer left
                idx += pol;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip inner left and right padding and the value width
                idx += pil + self.w + pir;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skipp outer right padding
                idx += por;

                line += 1;
            }

            // second and last line of full border
            // we skip the outer left padding values
            idx += pol;
            // we fill value length + inner padding right + left with border value
            for i in 0..pil + 1 + self.w + pir + 1 {
                lines[idx as usize] = Some(c);
                idx += 1;
            }
            // println!("{}: idx = {}, line = {}", line!(), idx, line,);
            // log_buf(&lines, wx, hx);
            // we skip the outer right padding
            idx += por;
            // second bordered line ends
            line += 1;
            assert_eq!(line + pob, pit + pot + self.h + pib + pob + 2);
            // println!("{}: idx = {}, line = {}", line!(), idx, line,);
            // log_buf(&lines, wx, hx);
        }
    }

    fn process_polyform(
        &self,
        c: char,
        lines: &mut Vec<Option<char>>,
        wx: u16,
        hx: u16,
        por: u16,
        pol: u16,
        pot: u16,
        pob: u16,
        pir: u16,
        pil: u16,
        pit: u16,
        pib: u16,
    ) {
    }
}

pub fn spread_padding(p: &Padding) -> [u16; 8] {
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
