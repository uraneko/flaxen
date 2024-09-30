use std::collections::HashMap;
use std::io::{StdoutLock, Write};
use std::str::Chars;

use crate::components::*;
use crate::space::{border::Border, padding::Padding};
use crate::themes::Style;

use super::spread_padding;

impl Container {
    /// wrapper around the render_border and render_value method calls
    pub fn render(&self, writer: &mut StdoutLock) {
        self.render_border(writer);
        self.render_value(writer);
    }

    /// renders only the items inside the container
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

    /// renders only the container border
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
    pub(crate) fn decorate(&self) -> [u16; 2] {
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
    pub(super) fn prepare(&self) -> (Vec<Option<char>>, [u16; 2]) {
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
                trcorner,
                tlcorner,
                brcorner,
                blcorner,
                rl,
                tb,
            } => self.process_polyform(
                trcorner, tlcorner, blcorner, brcorner, tb, rl, lines, wx, hx, por, pol, pot, pob,
                pir, pil, pit, pib,
            ),

            Border::Manual {
                tlcorner,
                trcorner,
                brcorner,
                blcorner,
                r0,
                rp,
                r1,
                l0,
                lp,
                l1,
                t0,
                tp,
                t1,
                b0,
                bp,
                b1,
            } => {
                self.process_manual(
                    tlcorner, trcorner, blcorner, brcorner, r0, rp, r1, l0, lp, l1, t0, tp, t1, b0,
                    bp, b1, lines, wx, hx, por, pol, pot, pob, pir, pil, pit, pib,
                );
            }
        }
    }

    fn process_none(
        &self,
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
        // first line of border
        // lines of outer top padding times number of cells in one line
        let mut idx = (pot + pit) * wx;
        let mut line = pot + pit;

        // handle the value lines
        // every iteration is a line
        while line < pot + pit + self.h {
            // skip inner left and right padding and the value width
            idx += pol + pil + self.w + pir + por;

            line += 1;
            // we have finished the value lines
        }

        // pass the bottom paddings
        idx += (pob + pib) * wx;

        assert_eq!(line + pob, pit + pot + self.h + pib + pob + 2);
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

    fn process_polyform(
        &self,
        trcorner: char,
        tlcorner: char,
        blcorner: char,
        brcorner: char,
        btb: char,
        blr: char,
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
        // first line of border
        // lines of padding times number of cells in one line
        let mut idx = pot * wx;
        let mut line = pot;
        // we skip the outer left padding values
        idx += pol;

        // we write the top left corner
        lines[idx as usize] = Some(tlcorner);
        idx += 1;

        // we fill value length + inner padding right + left with border top/bottom value,
        // excluding the top corners
        for i in 0..pil + self.w + pir {
            lines[idx as usize] = Some(btb);
            idx += 1;
        }

        // we write the top right corner
        lines[idx as usize] = Some(trcorner);
        idx += 1;

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
            lines[idx as usize] = Some(blr);
            idx += 1;
            // skip inner left and right padding and the value width
            idx += pil + self.w + pir;
            // border cell
            lines[idx as usize] = Some(blr);
            idx += 1;
            // skipp outer right padding
            idx += por;

            line += 1;
            // we are in front of the second full border line
        }

        // second and last line of full border
        // we skip the outer left padding values
        idx += pol;
        // we write the border bottom left corner value
        lines[idx as usize] = Some(blcorner);
        idx += 1;
        // we fill value length + inner padding right + left with border value
        for i in 0..pil + self.w + pir {
            lines[idx as usize] = Some(btb);
            idx += 1;
        }
        // we write the border bottom right value
        lines[idx as usize] = Some(brcorner);
        idx += 1;
        // we skip the outer right padding
        idx += por;
        // second bordered line ends
        line += 1;
        assert_eq!(line + pob, pit + pot + self.h + pib + pob + 2);
    }

    fn process_manual(
        &self,
        tlcorner: char,
        trcorner: char,
        blcorner: char,
        brcorner: char,
        r0: &str,
        rp: char,
        r1: &str,
        l0: &str,
        lp: char,
        l1: &str,
        t0: &str,
        tp: char,
        t1: &str,
        b0: &str,
        bp: char,
        b1: &str,
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
        // calculate border padding value len
        // we substract 2 at the end for the 2 corner values
        let mut bpt = wx - pol - por - t0.chars().count() as u16 - t1.chars().count() as u16 - 2;
        let mut bpb = wx - pol - por - b0.chars().count() as u16 - b1.chars().count() as u16 - 2;
        let mut bpr = hx - pot - pob - r0.chars().count() as u16 - r1.chars().count() as u16 - 2;
        let mut bpl = hx - pot - pob - l0.chars().count() as u16 - l1.chars().count() as u16 - 2;

        print!("wx: {}, hx: {}\r\n", wx, hx);
        print!("pot: {}, pol: {}, por: {}, pob: {}\r\n", pot, pol, por, pob);
        print!(
            "t0 len: {}, t1 len: {}\r\n",
            t0.chars().count(),
            t1.chars().count()
        );
        print!(
            "bpt = {} - {} - {} - {} - {} - 2 = {}",
            wx,
            pol,
            por,
            t0.chars().count(),
            t1.chars().count(),
            bpt
        );

        print!(
            "top: {} - bottom: {} - right: {} - left: {}\r\n",
            bpt, bpb, bpr, bpl
        );

        let bt = format!(
            "{}{}{}",
            t0,
            (0..bpt).into_iter().map(|_| tp).collect::<String>(),
            t1,
        );
        print!("bt {}=> \r\n{}\r\n\n", bt.chars().count(), bt);
        let mut bt = bt.chars();

        let br = format!(
            "{}{}{}",
            r0,
            (0..bpr).into_iter().map(|_| rp).collect::<String>(),
            r1,
        );
        print!("br => \r\n{}\r\n\n", br);
        let mut br = br.chars();

        let bl = format!(
            "{}{}{}",
            l0,
            (0..bpl).into_iter().map(|_| lp).collect::<String>(),
            l1,
        );
        print!("bl => \r\n{}\r\n\n", bl);
        let mut bl = bl.chars();

        let bb = format!(
            "{}{}{}",
            b0,
            (0..bpb).into_iter().map(|_| bp).collect::<String>(),
            b1,
        );
        print!("bb => \r\n{}\r\n\n", bb);
        let mut bb = bb.chars();

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

        // we write the top left corner
        lines[idx as usize] = Some(tlcorner);
        idx += 1;

        // log_buf(&lines, wx, hx);

        // write top border values
        while let Some(ch) = bt.next() {
            lines[idx as usize] = Some(ch);
            idx += 1;
        }

        // we write the top right corner
        lines[idx as usize] = Some(trcorner);
        idx += 1;

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
        while line < pot + 1 + pit + self.h + pib {
            // new line we skip padding outer left
            idx += pol;
            // left border cell
            lines[idx as usize] = bl.next();
            idx += 1;
            // skip inner left and right padding and the value width
            idx += pil + self.w + pir;
            // right border cell
            lines[idx as usize] = br.next();
            idx += 1;
            // skipp outer right padding
            idx += por;

            line += 1;
            // we are in front of the second full border line
        }
        // second and last line of full border
        // we skip the outer left padding values
        idx += pol;

        // we write the border bottom left corner value
        lines[idx as usize] = Some(blcorner);
        idx += 1;

        // write bottom border values
        while let Some(ch) = bb.next() {
            lines[idx as usize] = Some(ch);
            idx += 1;
        }

        // we write the border bottom right value
        lines[idx as usize] = Some(brcorner);
        idx += 1;

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
