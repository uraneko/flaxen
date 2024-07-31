#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{stdin, stdout, Read, Stdin, StdoutLock, Write};

fn tokenize(l: &mut String) -> Vec<&str> {
    l.trim_end_matches('\n')
        .trim()
        .split(' ')
        // .map(|s| s.to_string())
        .collect::<Vec<&str>>()
}

// TODO: get rid of crossterm dependency
// TODO: turn this module into a lib crate

#[derive(Debug)]
enum InputAction {
    PutChar(char),
    BackSpace,
    CRLF,
    MoveRight,
    MoveLeft,
    New,
    MoveEnd,
    // MoveRightWord,
    // MoveLeftWord,
    // RmRightWord,
    // RmLeftWord,
    MoveHome,
    HistoryPrev,
    HistoryNext,
}

enum Command {
    InputAction(InputAction),
    // Script,
    Exit(i32),
    None,
}

// TODO: change messy script save impl
// when ctrl + s is pressed user is prompted for a script name then on cr script is saved as name

pub fn flaxen() {
    let mut stdout = std::io::stdout().lock();

    let (mut input, mut history) = terminal_init();

    let mut user_input = String::new();

    '_main: loop {
        let cmd = keyboard();
        cmd.execute(&mut input, &mut history, &mut stdout, &mut user_input);
    }
}

impl Command {
    fn execute(&self, i: &mut Input, h: &mut History, sol: &mut StdoutLock<'_>, ui: &mut String) {
        match self {
            Command::InputAction(ia) => i.write(h, ia, sol, ui),
            Command::Exit(code) => Command::exit(*code),
            // Command::Script => Command::script(&h, &ui),
            Command::None => (),
        }
    }

    fn exit(code: i32) {
        std::process::exit(code);
    }

    fn script(h: &History, name: &str) {
        let script = h
            .log
            .iter()
            .map(|vec| vec.iter().collect::<String>())
            .filter(|l| &l[..7] != "script ")
            .fold(String::new(), |acc, x| acc + &x + "\r\n");

        std::fs::write(
            "resources/frieren/scripts/".to_string() + name + ".txt",
            script.into_bytes(),
        )
        .unwrap()
    }
}

use crossterm::event::{read as kbd_read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::enable_raw_mode;

use std::thread::{scope, Scope};

pub(crate) fn terminal_init() -> (Input, History) {
    _ = enable_raw_mode();

    // let in_buf_ex: [u8; 65_536] = [0; 65_536];

    // let in_buf: [u8; 8192] = [0; 8192];

    (Input::new(), History::new())
}

fn keyboard() -> Command {
    match kbd_read() {
        Ok(Event::Key(key_event)) => kbd_event(key_event),
        Err(e) => {
            eprintln!("read error\n{:?}", e);
            Command::None
        }
        _ => Command::None,
    }
}

fn kbd_event(key_event: KeyEvent) -> Command {
    match key_event.code {
        KeyCode::Enter if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::CRLF)
        }

        KeyCode::Backspace if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::BackSpace)
        }

        KeyCode::Up if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::HistoryPrev)
        }

        KeyCode::Down if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::HistoryNext)
        }

        KeyCode::Right if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::MoveRight)
        }

        KeyCode::Left if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::MoveLeft)
        }

        KeyCode::End if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::MoveEnd)
        }

        KeyCode::Home if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
            Command::InputAction(InputAction::MoveHome)
        }

        KeyCode::Char(c) => match c {
            'c' if key_event.modifiers == KeyModifiers::from_bits(0x2).unwrap() => Command::Exit(0),
            // 'h' if key_event.modifiers == KeyModifiers::from_bits(0x2).unwrap() => {
            //     Command::HistoryAction(HistoryAction::List)
            // }
            c if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
                Command::InputAction(InputAction::PutChar(c))
            }
            _ => Command::None,
        },

        _ => Command::None,
    }
}

#[derive(Debug)]
struct Input {
    values: Vec<char>,
    cursor: usize,
    debug_log: std::fs::File,
}

impl Input {
    fn new() -> Self {
        let mut i = Self {
            debug_log: std::fs::File::create("resources/logs/terminal/input").unwrap(),
            values: Vec::new(),
            cursor: 0,
        };
        i.log(&InputAction::New);

        i
    }

    fn write(
        &mut self,
        h: &mut History,
        ia: &InputAction,
        sol: &mut StdoutLock<'_>,
        ui: &mut String,
    ) {
        match ia {
            InputAction::New => (),
            InputAction::MoveRight => {
                if self.to_the_right() {
                    _ = sol.write(b"\x1b[C");
                }
            }

            InputAction::MoveLeft => {
                if self.to_the_left() {
                    _ = sol.write(b"\x1b[D");
                }
            }

            InputAction::BackSpace => {
                self.backspace();
                _ = sol.write(b"\x1b[2K");
                _ = sol.write(&[13]);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                _ = sol.write(&[13]);
                for _idx in 0..self.cursor {
                    _ = sol.write(b"\x1b[C");
                }
            }

            InputAction::CRLF => {
                self.cr_lf(h, ui);
                h.log(&ia);
                _ = sol.write(&[13, 10]);

                let mut tokens = tokenize(ui).into_iter();

                // TODO: tokens probably should be peekable in general
                // HACK: this is a wasteful hack
                // should be prompted in a popup buffer for the name

                ui.clear();
            }

            InputAction::PutChar(c) => {
                self.put_char(*c);
                _ = sol.write(b"\x1b[2K");
                _ = sol.write(&[13]);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                _ = sol.write(&[13]);
                for _idx in 0..self.cursor {
                    _ = sol.write(b"\x1b[C");
                }
            }

            InputAction::MoveEnd => match self.to_end() {
                0 => (),
                val => {
                    for _ in 0..val {
                        _ = sol.write(b"\x1b[C");
                    }
                }
            },

            InputAction::MoveHome => {
                if self.to_home() {
                    _ = sol.write(&[13]);
                }
            }

            InputAction::HistoryPrev => {
                if h.prev(&mut self.values) {
                    _ = sol.write(b"\x1b[2K");
                    _ = sol.write(&[13]);
                    self.cursor = self.values.len();
                    _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                }
                h.log(&ia);
            }

            InputAction::HistoryNext => {
                if h.next(&mut self.values) {
                    _ = sol.write(b"\x1b[2K");
                    _ = sol.write(&[13]);
                    self.cursor = self.values.len();
                    _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                }
                h.log(&ia);
            }
        }

        _ = sol.flush();
        self.log(&ia);
    }

    fn put_char(&mut self, c: char) {
        match self.values.is_empty() {
            true => {
                self.values.push(c);
                self.cursor += 1;
            }
            false => match self.cursor == self.values.len() {
                true => {
                    self.values.push(c);
                    self.cursor += 1;
                }

                false => {
                    self.values.insert(self.cursor, c);
                    self.cursor += 1;
                }
            },
        }
    }

    // TODO: shift cr registers input and sends it to command
    // WARN: do NOT touch the Input implementation...
    // the fns other than write are not to be touched

    fn cr_lf(&mut self, h: &mut History, user_input: &mut String) {
        h.push(self.values.clone());
        *user_input = self.values.clone().into_iter().collect::<String>();
        self.values = Vec::new();
        self.cursor = 0;
    }

    fn backspace(&mut self) {
        if self.values.is_empty() || self.cursor == 0 {
            return;
        }
        self.values.remove(self.cursor - 1);
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn to_the_right(&mut self) -> bool {
        if self.values.is_empty() || self.cursor == self.values.len() {
            return false;
        }
        self.cursor += 1;

        true
    }

    fn to_the_left(&mut self) -> bool {
        if self.values.is_empty() || self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;

        true
    }

    fn to_end(&mut self) -> usize {
        let diff = self.values.len() - self.cursor;
        if diff == 0 {
            return 0;
        }
        self.cursor = self.values.len();

        diff
    }

    fn to_home(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor = 0;

        true
    }

    fn log(&mut self, method: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?}\r\n",
                    method,
                    std::time::Instant::now(),
                    if self.cursor == 0 {
                        None
                    } else {
                        Some(self.cursor - 1)
                    },
                    if self.values.is_empty() || self.cursor == 0 {
                        None
                    } else {
                        Some(self.values[self.cursor - 1])
                    },
                    self.values,
                )
                .as_bytes(),
            )
            .unwrap();
    }
}

// NOTE: the cursor in both input and history does not point to the item it's on,
// but is alawys pointing at the item to the left
// basically cursor = 0 points at nothing and cursor = 4 points at eg. input[3]
// this logic is implemented in the functionality

#[derive(Debug)]
struct History {
    debug_log: std::fs::File,
    log: Vec<Vec<char>>,
    cursor: usize,
    temp: Option<Vec<char>>,
}

impl History {
    fn new() -> Self {
        let mut h = Self {
            debug_log: std::fs::File::create("resources/logs/terminal/history").unwrap(),
            log: Vec::new(),
            cursor: 0,
            temp: None,
        };
        h.log(&InputAction::New);

        h
    }

    // BUG: when input string is an empty value and history is visited
    // the temp logic breaks
    // FIXED using option<string> instead of string for temp

    fn prev(&mut self, value: &mut Vec<char>) -> bool {
        if self.cursor == 0 {
            return false;
        }

        if self.temp.is_none() || self.cursor == self.log.len() {
            self.temp = Some(value.clone()); // temporarily keep input val
        }

        *value = self.log[self.cursor - 1].clone();
        self.cursor -= 1;

        true
    }

    fn next(&mut self, value: &mut Vec<char>) -> bool {
        if self.cursor == self.log.len() {
            return false;
        }

        if self.cursor + 1 == self.log.len() {
            *value = self.temp.as_ref().unwrap().clone();
        } else {
            *value = self.log[self.cursor + 1].clone();
        }
        self.cursor += 1;

        true
    }

    fn push(&mut self, value: Vec<char>) {
        if value.iter().filter(|c| **c != ' ').count() > 0 && !self.log.contains(&value) {
            self.log.push(value);
        }
        self.temp = None;
        self.cursor = self.log.len();
    }

    fn log(&mut self, method: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?}\r\n",
                    method,
                    std::time::Instant::now(),
                    if self.cursor == 0 {
                        None
                    } else {
                        Some(self.cursor - 1)
                    },
                    if self.log.is_empty() || self.cursor == 0 {
                        None
                    } else {
                        Some(self.log[self.cursor - 1].clone())
                    },
                    self.log,
                )
                .as_bytes(),
            )
            .unwrap();
    }
}
// input:
// starts empty,
// only writing is possible: value.push(char)
// that unlocks:
// movement to the right/left
// inserting char at any position inside input value
// backspace erasure of any position inside input value char

// TODO: program prompt
