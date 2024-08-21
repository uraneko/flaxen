use crossterm::event::read as kbd_read;
use std::io::BufRead;

use std::io::Read;
use std::io::Write;

#[derive(Debug)]
enum KbdInput {
    ControlChar(ControlChar),
    Char(char),
    Error(std::io::Error),
}

#[derive(Debug)]
enum ControlChar {
    NUL,
    BEL,
    BS,
    TAB,
    LF,
    CR,
    ESC,
    Deprecated(u8),
}

trait UTF8Decoder
where
    Self: Sized,
{
    fn decoder(input: Vec<Self>) -> String;
    fn point(code: u8) -> Self;
}

// NOTE: use recursion in decoder
impl UTF8Decoder for KbdInput {
    fn decoder(input: Vec<Self>) -> String {
        "".to_string()
    }
    fn point(code: u8) -> Self {
        match code.ge(&0) && code.lt(&31) || code == 127 {
            true => match code {
                0 => Self::ControlChar(ControlChar::NUL),
                7 => Self::ControlChar(ControlChar::BEL),
                8 | 127 => Self::ControlChar(ControlChar::BEL),
                10 => Self::ControlChar(ControlChar::LF),
                9 => Self::ControlChar(ControlChar::TAB),
                13 => Self::ControlChar(ControlChar::CR),
                27 => Self::ControlChar(ControlChar::ESC),
                code => Self::ControlChar(ControlChar::Deprecated(code)),
            },
            false => panic!(),
        }
    }
}

trait KbdEvent
where
    Self: std::iter::IntoIterator,
{
    fn resolve(&self) -> KbdInput;

    fn residual(&self) -> Vec<u8>;
}

impl KbdEvent for [u8; 8] {
    fn resolve(&self) -> KbdInput {
        match self.len() {
            // if only one byte:
            // it could be an ascii, an ansi or a utf8 byte
            1 => match self[0] {
                // ascii control byte
                code if code.ge(&0) && code.lt(&32) || code.eq(&127) => match code {
                    0 => KbdInput::ControlChar(ControlChar::NUL),
                    7 => KbdInput::ControlChar(ControlChar::BEL),
                    8 | 127 => KbdInput::ControlChar(ControlChar::BEL),
                    10 => KbdInput::ControlChar(ControlChar::LF),
                    9 => KbdInput::ControlChar(ControlChar::TAB),
                    13 => KbdInput::ControlChar(ControlChar::CR),
                    27 => KbdInput::ControlChar(ControlChar::ESC),
                    code => KbdInput::ControlChar(ControlChar::Deprecated(code)),
                },
                // ascii char byte
                code if code.lt(&127) && code.gt(&31) => KbdInput::Char(code as char),

                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn residual(&self) -> Vec<u8> {
        {
            let mut buf = self.iter().map(|b| *b).rev().collect::<Vec<u8>>();
            loop {
                if buf[0] == 0 {
                    buf.remove(0);
                } else {
                    break buf.into_iter().rev().collect();
                }
            }
        }
    }
}

fn decode(bytes: &[u8; 8]) -> KbdInput {
    // NOTE: should check for what kind of utf8 first, i.e., 1, 2, 3 or 4 bytes
    if bytes[1] == 0 && bytes[2] == 0 {
        // ascii one byte or maybe has modifiers
        if bytes[0] & 128 == 0 {
            // control char
            if bytes[0] & (32 | 64) == 0 {
                KbdInput::ControlChar(match bytes[0] {
                    0 => ControlChar::NUL,
                    7 => ControlChar::BEL,
                    8 | 127 => ControlChar::BEL,
                    10 => ControlChar::LF,
                    9 => ControlChar::TAB,
                    13 => ControlChar::CR,
                    27 => ControlChar::ESC,
                    code => ControlChar::Deprecated(code),
                })
                // ascii printable char, no modifiers
            } else if bytes[0] & (126) != 0 {
                KbdInput::Char(bytes[0] as char)
            } else {
                panic!()
            }

            // either ascii with modifiers or 2 bytes utf8
        } else if bytes[1] != 0 && bytes[2] == 0 {
            panic!()
        } else {
            panic!()
        }
    } else {
        panic!()
    }
}

fn main() {
    let original = ragout_assistant::raw_mode::raw_mode();

    let mut buf: [u8; 8] = [0; 8];
    let mut sol = std::io::stdout().lock();

    let mut input: Vec<u8> = vec![];
    let mut si = std::io::stdin().lock();

    // getchar just gets what was pressed
    // it does not detect the press
    // getchar();

    let fps = 60;
    let counter = 1000 / fps;
    loop {
        // crossterm's kbd_read (real name event::read()) did not fare any better that my
        // simple stdin reading at detecting the CTRL modifier when used with the '.' char
        // // this block inside the loop is the read keyboard key press

        // print!("\r\n{:?}", kbd_read());
        // sol.flush();

        std::thread::sleep(std::time::Duration::from_millis(counter));

        if let Ok(n) = si.read(&mut buf) {
            if n > 0 {
                print!(
                    "\r\n{:?}\r\n{:?}",
                    buf,
                    decode(&buf) // std::str::from_utf8(&buf).unwrap().trim_end_matches('\0')
                );
                // print!("\r\nthe full input is: {:?}", input);
                _ = sol.flush();

                match buf.iter().filter(|b| **b != 0).count() == 0 {
                    true => input.push(0),
                    false => input.append(&mut {
                        let mut buf = buf.iter().map(|b| *b).rev().collect::<Vec<u8>>();
                        loop {
                            if buf[0] == 0 {
                                buf.remove(0);
                            } else {
                                break buf.into_iter().rev().collect();
                            }
                        }
                    }),
                }
                buf = Default::default();
            }
        }
    }
}
