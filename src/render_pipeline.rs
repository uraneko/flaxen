use crate::object_tree::*;
use crate::space_awareness::{Border, Padding};

use std::io::{StdoutLock, Write};

// trait Renderer {
//     fn render(&self, writer: &mut StdoutLock) {}
//     fn clear(&self, writer: &mut StdoutLock) {}
//     fn place(&mut self, writer: &mut StdoutLock, x: u16, y: u16) {}
//     fn prepare(&self) -> (Vec<Option<char>>, [u16; 2]) {
//         (vec![], [0, 0])
//     }
//     fn decorate(&self) -> [u16; 2] {
//         [0, 0]
//     }
// }

// NOTE: an object can not be initialized unless
// its id is valid
// its dimensions are valid, including overlay

impl Term {
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
    /// assumes that Term::clear() has been used before hand to prepare the terminal display for the
    /// rendering
    // this method doesn't work at all
    pub fn render(&self, writer: &mut StdoutLock) {
        let cells = self.prepare();

        let mut s = String::new();

        let mut line = 0;
        let mut idx = 0;

        // BUG: we need all Some and None
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

        // BUG: t.x0 and t.y0 need to have c.border and c.paddings
        // added to them
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
                // new line we skip padding outer left
                idx += pol;
                // border cell
                lines[idx as usize] = Some(c);
                idx += 1;
                // skip inner left padding
                idx += pil;
                // write values
                // BUG: doesn't handle multi lined values
                for vi in 0..self.w as usize {
                    lines[idx as usize] = Some(self.value[vi]);
                    idx += 1;
                }
                // skipp inner right padding
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

fn spread_padding(p: &Padding) -> [u16; 8] {
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
