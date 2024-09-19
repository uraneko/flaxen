use std::collections::HashMap;
use std::io::{StdoutLock, Write};
use std::str::Chars;

use crate::components::*;
use crate::space::{Border, Padding};
use crate::themes::Style;

use super::spread_padding;

impl Text {
    /// wrapper around the render_border and render_value method calls
    pub fn render(&self, writer: &mut StdoutLock) {
        self.render_border(writer);
        self.render_value(writer);
    }

    /// renders only the text border
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

    /// renders only the text value
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

    // this should be used inside the container prepare method
    pub(super) fn prepare(&self) -> (Vec<Option<char>>, [u16; 2]) {
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
            // border cell, write border left/right value
            lines[idx as usize] = Some(blr);
            idx += 1;
            // skip inner left and right padding and the value len
            idx += pil + self.w + pir;
            // border cell, write border left/right value
            lines[idx as usize] = Some(blr);
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
            lines[idx as usize] = Some(blr);
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
            lines[idx as usize] = Some(blr);
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

        let bt = format!(
            "{}{}{}",
            t0,
            (0..bpt).into_iter().map(|_| tp).collect::<String>(),
            t1,
        );
        let mut bt = bt.chars();

        let br = format!(
            "{}{}{}",
            r0,
            (0..bpr).into_iter().map(|_| rp).collect::<String>(),
            r1,
        );
        let mut br = br.chars();

        let bl = format!(
            "{}{}{}",
            l0,
            (0..bpl).into_iter().map(|_| lp).collect::<String>(),
            l1,
        );
        let mut bl = bl.chars();

        let bb = format!(
            "{}{}{}",
            b0,
            (0..bpb).into_iter().map(|_| bp).collect::<String>(),
            b1,
        );
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

        // log_buf(&lines, wx, hx);

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
        while line < v0 {
            // new line we skip padding outer left
            idx += pol;
            // border cell, write border left/right value
            lines[idx as usize] = bl.next();
            idx += 1;
            // skip inner left and right padding and the value len
            idx += pil + self.w + pir;
            // border cell, write border left/right value
            lines[idx as usize] = br.next();
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
            lines[idx as usize] = bl.next();
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
            lines[idx as usize] = br.next();
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
            lines[idx as usize] = bl.next();
            idx += 1;
            // skip inner left and right padding and the value width
            idx += pil + self.w + pir;
            // border cell
            lines[idx as usize] = br.next();
            idx += 1;
            // skipp outer right padding
            idx += por;

            line += 1;
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
