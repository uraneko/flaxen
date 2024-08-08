#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{stdin, stdout, Read, Stdin, StdoutLock, Write};

// TODO: get rid of crossterm dependency
// TODO: render graphics
// TODO: add a prompt
// raw mode:
// from [https://www.reddit.com/r/rust/comments/1d3ofwo/raw_mode_in_terminal/]
// comment [https://www.reddit.com/r/rust/comments/1d3ofwo/comment/l68vr45/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button]
// If you don’t want to use any external crates
// you need to create exetrns for C functions from unistd.h
// and possibly some other places.
// Specifically to enable raw mode you need tcgetattr and tcsetattr functions.

// If you’re willing to accept external crates which provide low-level wrappers for C functions
// than you libc and nix crate will provide all the functions and types.
//
//

#[derive(Debug)]
enum InputAction {
    PutChar(char),
    BackSpace,
    CRLF,
    MoveRight,
    MoveLeft,
    New,
    MoveEnd,
    MoveRightJump,
    MoveLeftJump,
    ClearLine,
    ClearRight,
    ClearLeft,
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

pub fn tokenize(l: &mut String) -> Vec<&str> {
    if l.is_empty() {
        return vec![];
    }

    l.trim_end_matches('\n')
        .trim()
        .split(' ')
        .collect::<Vec<&str>>()

    // assert!({
    //     let mut t = tokens.clone();
    //     t.dedup();
    //     t != vec![""]
    // });
}

pub fn init() -> (std::io::StdoutLock<'static>, Input, History, String) {
    _ = enable_raw_mode();

    (
        std::io::stdout().lock(),
        Input::new(),
        History::new(),
        String::new(),
    )
}

pub fn run<'a>(
    input: &mut Input,
    history: &mut History,
    stdout: &mut std::io::StdoutLock<'static>,
    user_input: &'a mut String,
) -> Vec<&'a str> {
    let cmd = keyboard();
    cmd.execute(input, history, stdout, user_input);

    tokenize(user_input)
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

        KeyCode::Backspace if key_event.modifiers == KeyModifiers::from_bits(0x4).unwrap() => {
            Command::InputAction(InputAction::ClearLine)
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

        KeyCode::Right if key_event.modifiers == KeyModifiers::from_bits(0x4).unwrap() => {
            Command::InputAction(InputAction::MoveRightJump)
        }
        KeyCode::Left if key_event.modifiers == KeyModifiers::from_bits(0x4).unwrap() => {
            Command::InputAction(InputAction::MoveLeftJump)
        }

        KeyCode::Char(c) => match c {
            'c' if key_event.modifiers == KeyModifiers::from_bits(0x2).unwrap() => Command::Exit(0),
            // 'h' if key_event.modifiers == KeyModifiers::from_bits(0x2).unwrap() => {
            //     Command::HistoryAction(HistoryAction::List)
            // }
            'r' if key_event.modifiers == KeyModifiers::from_bits(0x4).unwrap() => {
                Command::InputAction(InputAction::ClearRight)
            }
            'l' if key_event.modifiers == KeyModifiers::from_bits(0x4).unwrap() => {
                Command::InputAction(InputAction::ClearLeft)
            }
            c if key_event.modifiers == KeyModifiers::from_bits(0x0).unwrap() => {
                Command::InputAction(InputAction::PutChar(c))
            }
            _ => Command::None,
        },

        _ => Command::None,
    }
}

#[derive(Debug)]
pub struct Input {
    values: Vec<char>,
    cursor: usize,
    #[cfg(debug_assertions)]
    debug_log: std::fs::File,
}

impl Input {
    fn new() -> Self {
        let mut i = Self {
            #[cfg(debug_assertions)]
            debug_log: std::fs::File::create("resources/logs/terminal/input").unwrap(),
            values: Vec::new(),
            cursor: 0,
        };
        #[cfg(debug_assertions)]
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

            InputAction::ClearLine => {
                self.clear_line();
                _ = sol.write(b"\x1b[2K");
                _ = sol.write(&[13]);
            }

            InputAction::ClearRight => {
                self.clear_right();
                _ = sol.write(b"\x1b[0K");

                // _ = sol.write(b"\x1b[2K");
                // _ = sol.write(&[13]);
                // _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
            }

            InputAction::ClearLeft => {
                self.clear_left();
                _ = sol.write(b"\x1b[2K");
                _ = sol.write(&[13]);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                _ = sol.write(&[13]);
            }

            InputAction::CRLF => {
                self.cr_lf(h, ui);
                #[cfg(debug_assertions)]
                h.log(&ia);
                _ = sol.write(&[13, 10]);

                // TODO: tokens probably should be peekable in general
                // HACK: this is a wasteful hack
                // should be prompted in a popup buffer for the name
            }

            InputAction::PutChar(c) => {
                self.put_char(*c);
                // _ = sol.write(b"\x1b[31;1;4m");
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

            InputAction::MoveRightJump => {
                self.to_right_jump();
                _ = sol.write(&[13]);
                for _idx in 0..self.cursor {
                    _ = sol.write(b"\x1b[C");
                }
            }

            InputAction::MoveLeftJump => {
                self.to_left_jump();
                _ = sol.write(&[13]);
                for _idx in 0..self.cursor {
                    _ = sol.write(b"\x1b[C");
                }
            }

            InputAction::HistoryPrev => {
                if h.prev(&mut self.values) {
                    _ = sol.write(b"\x1b[2K");
                    _ = sol.write(&[13]);
                    self.cursor = self.values.len();
                    _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                }
                #[cfg(debug_assertions)]
                h.log(&ia);
            }

            InputAction::HistoryNext => {
                if h.next(&mut self.values) {
                    _ = sol.write(b"\x1b[2K");
                    _ = sol.write(&[13]);
                    self.cursor = self.values.len();
                    _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                }
                #[cfg(debug_assertions)]
                h.log(&ia);
            }
        }

        _ = sol.flush();
        #[cfg(debug_assertions)]
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
        h.push(self.values.to_vec());
        *user_input = self.values.drain(..).collect::<String>();
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

    fn clear_line(&mut self) {
        self.cursor = 0;
        self.values.clear();
    }

    fn clear_right(&mut self) {
        for _ in self.cursor..self.values.len() {
            self.values.pop();
        }
    }

    fn clear_left(&mut self) {
        for _ in 0..self.cursor {
            self.values.remove(0);
        }
        self.cursor = 0;
    }

    const STOPPERS: [char; 11] = ['/', ' ', '-', '_', ',', '"', '\'', ';', ':', '.', ','];

    fn to_right_jump(&mut self) {
        if self.cursor == self.values.len() {
            return;
        }

        match self.values[if self.cursor + 1 < self.values.len() {
            self.cursor + 1
        } else {
            self.cursor
        }] == ' '
        {
            true => {
                while self.cursor + 1 < self.values.len() && self.values[self.cursor + 1] == ' ' {
                    self.cursor += 1;
                }
            }
            false => {
                while self.cursor + 1 < self.values.len()
                    && !Self::STOPPERS.contains(&self.values[self.cursor + 1])
                {
                    self.cursor += 1;
                }
                self.cursor += 1;
            }
        }
    }

    fn to_left_jump(&mut self) {
        if self.cursor == 0 {
            return;
        }

        match self.values[self.cursor - 1] == ' ' {
            true => {
                while self.cursor > 0 && self.values[self.cursor - 1] == ' ' {
                    self.cursor -= 1;
                }
            }
            false => {
                while self.cursor > 1 && !Self::STOPPERS.contains(&self.values[self.cursor - 1]) {
                    self.cursor -= 1;
                }
                self.cursor -= 1;
            }
        }
    }

    #[cfg(debug_assertions)]
    fn log(&mut self, method: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?}\r\n",
                    method,
                    std::process::Command::new("date")
                        .arg("+\"%H:%M:%S:%N\"")
                        .output()
                        .expect("couldnt get time from linux command 'date'")
                        .stdout
                        .into_iter()
                        .map(|u| u as char)
                        .collect::<String>()
                        .replacen("\"", "", 2)
                        .trim_end_matches("\n"),
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
pub struct History {
    #[cfg(debug_assertions)]
    debug_log: std::fs::File,
    log: Vec<Vec<char>>,
    cursor: usize,
    temp: Option<Vec<char>>,
}

impl History {
    fn new() -> Self {
        let mut h = Self {
            #[cfg(debug_assertions)]
            debug_log: std::fs::File::create("resources/logs/terminal/history").unwrap(),
            log: Vec::new(),
            cursor: 0,
            temp: None,
        };
        #[cfg(debug_assertions)]
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

    #[cfg(debug_assertions)]
    fn log(&mut self, method: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?}\r\n",
                    method,
                    std::process::Command::new("date")
                        .arg("+\"%H:%M:%S:%N\"")
                        .output()
                        .expect("couldnt get time from linux command 'date'")
                        .stdout
                        .into_iter()
                        .map(|u| u as char)
                        .collect::<String>()
                        .replacen("\"", "", 2)
                        .trim_end_matches("\n"),
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
