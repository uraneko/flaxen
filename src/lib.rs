//! # ragout
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::{stdin, stdout, Read, Stdin, StdoutLock, Write};

// TODO: get rid of crossterm dependency
// TODO: render graphics
// TODO: option to start  in alternate screen
// would use "\e[?1049h" to enter alternate screen
// then use  "\e[?1049l" to exit alternate screen when exiting program
// raw mode:
// from [https://www.reddit.com/r/rust/comments/1d3ofwo/raw_mode_in_terminal/]
// comment [https://www.reddit.com/r/rust/comments/1d3ofwo/comment/l68vr45/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button]
// "If you don’t want to use any external crates
// you need to create exetrns for C functions from unistd.h
// and possibly some other places.
// Specifically to enable raw mode you need tcgetattr and tcsetattr functions.

// If you’re willing to accept external crates which provide low-level wrappers for C functions
// than you libc and nix crate will provide all the functions and types."
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

pub fn init(prompt: &str) -> (std::io::StdoutLock<'static>, Input, History, String) {
    _ = enable_raw_mode();

    let mut sol = std::io::stdout().lock();
    let i = Input::new(prompt);
    i.write_prompt(&mut sol);
    _ = sol.flush();

    (sol, i, History::new(), String::new())
}

pub fn run(
    input: &mut Input,
    history: &mut History,
    stdout: &mut std::io::StdoutLock<'static>,
    user_input: &mut String,
) -> String {
    let cmd = keyboard();
    cmd.execute(input, history, stdout, user_input);

    user_input.drain(..).collect::<String>()
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
            "resources/app/scripts/".to_string() + name + ".txt",
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

        KeyCode::Right if key_event.modifiers == KeyModifiers::from_bits(0x6).unwrap() => {
            Command::InputAction(InputAction::ClearRight)
        }

        KeyCode::Left if key_event.modifiers == KeyModifiers::from_bits(0x6).unwrap() => {
            Command::InputAction(InputAction::ClearLeft)
        }

        KeyCode::Char(c) => match c {
            'c' if key_event.modifiers == KeyModifiers::from_bits(0x2).unwrap() => Command::Exit(0),
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
    prompt: String,
}

impl Input {
    fn new(prompt: &str) -> Self {
        let mut i = Self {
            #[cfg(debug_assertions)]
            debug_log: std::fs::File::create("resources/logs/terminal/input").unwrap_or_else(
                |_| {
                    std::fs::create_dir_all("resources/logs/terminal").unwrap();
                    std::fs::File::create("resources/logs/terminal/input").unwrap()
                },
            ),
            values: Vec::new(),
            cursor: 0,
            prompt: prompt.to_owned(),
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
                self.write_prompt(sol);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                self.sync_cursor(sol);
            }

            InputAction::ClearLine => {
                self.clear_line();
                self.write_prompt(sol);
            }

            InputAction::ClearRight => {
                self.clear_right();
                _ = sol.write(b"\x1b[0K");
                self.sync_cursor(sol);
            }

            InputAction::ClearLeft => {
                self.clear_left();
                self.write_prompt(sol);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                self.sync_cursor(sol);
            }

            InputAction::CRLF => {
                self.cr_lf(h, ui);
                #[cfg(debug_assertions)]
                h.log(&ia);
                _ = sol.write(&[13, 10]);
                self.write_prompt(sol);

                // TODO: tokens probably should be peekable in general
                // HACK: this is a wasteful hack
                // should be prompted in a popup buffer for the name
            }

            InputAction::PutChar(c) => {
                self.put_char(*c);
                // _ = sol.write(b"\x1b[31;1;4m");
                self.write_prompt(sol);
                _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                self.sync_cursor(sol);
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
                    for _ in 0..self.prompt.len() {
                        _ = sol.write(b"\x1b[C");
                    }
                    // OR
                    // self.write_prompt(sol);
                }
            }

            InputAction::MoveRightJump => {
                self.to_right_jump();
                _ = sol.write(&[13]);
                self.sync_cursor(sol);
            }

            InputAction::MoveLeftJump => {
                self.to_left_jump();
                _ = sol.write(&[13]);
                self.sync_cursor(sol);
            }

            InputAction::HistoryPrev => {
                if h.prev(&mut self.values) {
                    self.write_prompt(sol);
                    self.cursor = self.values.len();
                    _ = sol.write(&self.values.iter().map(|c| *c as u8).collect::<Vec<u8>>());
                }
                #[cfg(debug_assertions)]
                h.log(&ia);
            }

            InputAction::HistoryNext => {
                if h.next(&mut self.values) {
                    self.write_prompt(sol);
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
}

impl Input {
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

    // PRIORITY HIGH:
    // TODO: add prompt (wip)
    // TODO: add documentation for the whole crate (branch docs)
    //

    // TODO: shift cr registers input and sends it to command; aka multi line input
    // WARN: do NOT touch this Input implementation
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
        if self.cursor > 0 {
            self.values.remove(self.cursor - 1);
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
        if diff > 0 {
            self.cursor = self.values.len();
        }

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
            debug_log: std::fs::File::create("resources/logs/terminal/history").unwrap_or_else(
                |_| {
                    std::fs::create_dir_all("resources/logs/terminal").unwrap();
                    std::fs::File::create("resources/logs/terminal/history").unwrap()
                },
            ),
            log: Vec::new(),
            cursor: 0,
            temp: None,
        };
        #[cfg(debug_assertions)]
        h.log(&InputAction::New);

        h
    }

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

#[cfg(test)]
mod test_input {
    use super::{init, run, History, Input};
    use std::io::Write;

    #[test]
    fn test_put_char() {
        let mut i = Input::new("");

        let mut idx = 0;
        ['p', 'i', 'k', 'a'].into_iter().for_each(|c| {
            i.put_char(c);
            idx += 1;

            assert_eq!(i.values[i.cursor - 1], c);
            assert_eq!(idx, i.cursor);
        })
    }

    #[test]
    fn test_backspace() {
        let mut i = Input::new("");

        let input = "pikatchino";
        input.chars().into_iter().for_each(|c| i.put_char(c));

        i.backspace();

        assert!({ i.cursor == input.len() - 1 && i.values[i.cursor - 1] == 'n' });
    }

    #[test]
    fn test_to_end() {
        let mut i = Input::new("");

        "pikatchaa".chars().into_iter().for_each(|c| i.put_char(c));
        // cursor is by default at end, but we still move it to end
        i.to_end();

        assert!({ i.cursor == 9 && i.values[i.cursor - 1] == 'a' });

        // now we test moving to end from somewhere else
        i.to_the_left();
        i.to_the_left();
        i.to_end();

        assert!({ i.cursor == 9 && i.values[i.cursor - 1] == 'a' });

        // and finally, moving to end from home (first cell in line)
        i.to_home();
        i.to_end();

        assert!({ i.cursor == 9 && i.values[i.cursor - 1] == 'a' });
    }

    #[test]
    fn test_to_home() {
        let mut i = Input::new("");

        "pikatchuu".chars().into_iter().for_each(|c| i.put_char(c));
        i.to_home();

        assert!({ i.cursor == 0 && i.values[i.cursor] == 'p' });
    }

    #[test]
    fn test_to_the_right() {
        let mut i = Input::new("");

        "pikatchau".chars().into_iter().for_each(|c| i.put_char(c));
        i.to_the_left();
        i.to_the_left();

        assert_eq!(i.values[i.cursor - 1], 'h');
        assert_eq!(i.cursor, "pikatchau".len() - 2);
    }

    #[test]
    fn test_to_the_left() {
        let mut i = Input::new("");

        "pikatchau".chars().into_iter().for_each(|c| i.put_char(c));
        i.to_home();
        i.to_the_right();
        i.to_the_right();

        assert_eq!(i.values[i.cursor], 'k');
        assert_eq!(i.cursor, 2);
    }

    #[test]
    fn test_cr_lf() {
        let mut i = Input::new("");
        let mut h = History::new();
        let mut user_input = String::new();

        "pikatcharu".chars().into_iter().for_each(|c| i.put_char(c));

        i.cr_lf(&mut h, &mut user_input);

        assert_eq!(
            h.log[0],
            "pikatcharu".chars().into_iter().collect::<Vec<char>>()
        );
        assert!(i.values.is_empty());
        assert_eq!(i.cursor, 0);
    }

    #[test]
    fn test_clear_line() {
        let mut i = Input::new("");

        "pikauchi".chars().into_iter().for_each(|c| i.put_char(c));

        assert!({ i.cursor == "pikauchi".len() && i.values[i.cursor - 1] == 'i' });

        i.clear_line();
        assert!(i.values.is_empty());
        assert_eq!(i.cursor, 0);
    }

    #[test]
    fn test_clear_right() {
        let mut i = Input::new("");

        "pikatchiatto"
            .chars()
            .into_iter()
            .for_each(|c| i.put_char(c));
        (0..4).for_each(|_| {
            i.to_the_left();
        });

        i.clear_right();
        assert_eq!(i.values.iter().map(|c| *c).collect::<String>(), "pikatchi");
    }

    #[test]
    fn test_clear_left() {
        let mut i = Input::new("");

        "pikatchiatto"
            .chars()
            .into_iter()
            .for_each(|c| i.put_char(c));
        (0..4).for_each(|_| {
            i.to_the_left();
        });

        i.clear_left();
        assert_eq!(i.values.iter().map(|c| *c).collect::<String>(), "atto");
    }
}

impl Input {
    fn overwrite_prompt(&mut self, new_prompt: &str) {
        self.prompt.clear();
        self.prompt.push_str(new_prompt);
    }

    fn write_prompt(&self, sol: &mut StdoutLock) {
        _ = sol.write(b"\x1b[2K");
        _ = sol.write(&[13]);
        _ = sol.write(
            &self
                .prompt
                .chars()
                .into_iter()
                .map(|c| c as u8)
                .collect::<Vec<u8>>(),
        );
        // _ = sol.write(b" ");
        _ = sol.flush();
    }

    fn sync_cursor(&self, sol: &mut StdoutLock) {
        _ = sol.write(&[13]);
        for _idx in 0..self.prompt.len() + self.cursor {
            _ = sol.write(b"\x1b[C");
        }
    }
}

// #[cfg(test)]
// mod test_prompt {}
