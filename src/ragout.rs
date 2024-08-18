#![cfg(feature = "lib")]
use std::io::{StdoutLock, Write};
use std::os::fd::AsRawFd;

///
/// Enables terminal raw mode and initializes the necessary variables for behaving in the raw mode.
///
/// Takes a [`&str`] for the shell prompt (give "" for no prompt) and a bool for the option of running in the terminal alternate screen (give true to run your cli program in alternate screen)
/// # Errors
/// Does NOT return errors and never panics
///
/// # Example
///
/// Basic usage
///
/// ```
/// use ragout::{init, run};
///
/// fn main() {
///     // enter raw mode and initialize necessary variables
///     // the string literal argument will be the value of the prompt
///     let (mut sol, mut i, mut h, mut ui) = init("some prompt 🐱 ", true);
///
///     'main: loop {
///         let input = run(&mut i, &mut h, &mut sol, &mut ui);
///         if !input.is_empty() {
///             // do some stuff with the user input
///         }
///     }
/// }
/// ```
pub use ragout_assistant::init;
use ragout_assistant::{DebugLog, Writer};
use ragout_assistant::{History, Input};

// CONTEMPLATE: TODO: Input and History do not have to be public to the user of the lib
// TODO: get rid of crossterm dependency
// TODO: render graphics

// raw mode:
// you need to create exetrns for C functions from unistd.h
// Specifically to enable raw mode you need tcgetattr and tcsetattr functions.

// move the logs in History::new and Input::new to this fn
// since they can't stay there due to design limitations
// #[cfg(any(debug_assertions, feature = "debug_logs"))]
// pub fn log_init(i: &mut Input, h: &mut History) {
//     i.log(&InputAction::New);
//     h.log(&InputAction::New);
// }

/// catches the key event, executes the matching Command then returns the user input if any
///
/// # Errors
///
/// Can not panic! or return Error if crate feature 'lib' is enabled
/// If crate feature 'custom_events' is enabled, and custom events are provided, this function may panic! if there is an error in those custom events closures
///
/// # Example
///
/// Basic usage:
///
/// ```
/// use ragout::{init, run};
///
/// fn main() {
///     // enter raw mode and initialize necessary variables
///     // the string literal argument will be the value of the prompt
///     let (mut sol, mut i, mut h, mut ui) = init("some prompt 🐱 ", true);
///
///     'main: loop {
///         let input = run(&mut i, &mut h, &mut sol, &mut ui);
///         if !input.is_empty() {
///             // do some stuff with the user input
///         }
///     }
/// }
///
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

#[derive(Debug)]
enum InputAction {
    PutChar(char),
    BackSpace,
    CRLF,
    MoveRight,
    MoveLeft,
    // New,
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

impl Command {
    fn execute(&self, i: &mut Input, h: &mut History, sol: &mut StdoutLock<'_>, ui: &mut String) {
        match self {
            Command::InputAction(ia) => i.write(h, ia, sol, ui),
            Command::Exit(code) => {
                if i.alt_screen {
                    _ = sol.write(b"\x1b[?1049l");
                }
                Command::exit(*code)
            }
            // Command::Script => Command::script(&h, &ui),
            Command::None => (),
        }
    }

    fn exit(code: i32) {
        std::process::exit(code);
    }

    // fn script(h: &History, name: &str) {
    //     let script = h
    //         .values
    //         .iter()
    //         .map(|vec| vec.iter().collect::<String>())
    //         .filter(|l| &l[..7] != "script ")
    //         .fold(String::new(), |acc, x| acc + &x + "\r\n");
    //
    //     std::fs::write(
    //         "resources/app/scripts/".to_string() + name + ".txt",
    //         script.into_bytes(),
    //     )
    //     .unwrap()
    // }
}

use crossterm::event::{read as kbd_read, Event, KeyCode, KeyEvent, KeyModifiers};

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

impl Writer<InputAction> for Input {
    fn write(
        &mut self,
        h: &mut History,
        ia: &InputAction,
        sol: &mut StdoutLock<'_>,
        ui: &mut String,
    ) {
        match ia {
            // InputAction::New => (),
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
                self.sync_cursor(sol);
            }
            InputAction::CRLF => {
                self.cr_lf(h, ui);
                #[cfg(any(debug_assertions, feature = "debug_logs"))]
                h.log(&ia);
                _ = sol.write(&[13, 10]);
                self.write_prompt(sol);

                // TODO: tokens probably should be peekable in general
                // HACK: this is a wasteful hack
                // should be prompted in a popup buffer for the name
            }

            InputAction::PutChar(c) => {
                self.put_char(*c);
                self.write_prompt(sol);
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
                    for _ in 0..self.prompt.chars().count() + 1 {
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
                }
                #[cfg(any(debug_assertions, feature = "debug_logs"))]
                h.log(&ia);
            }

            InputAction::HistoryNext => {
                if h.next(&mut self.values) {
                    self.write_prompt(sol);
                    self.cursor = self.values.len();
                }
                #[cfg(any(debug_assertions, feature = "debug_logs"))]
                h.log(&ia);
            }
        }

        _ = sol.flush();
        #[cfg(any(debug_assertions, feature = "debug_logs"))]
        self.log(&ia);
    }
}

#[cfg(any(debug_assertions, feature = "debug_logs"))]
impl DebugLog<InputAction> for Input {
    fn log(&mut self, event: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?}\r\n",
                    event,
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

    fn dl_rfd(&self) -> i32 {
        self.debug_log.as_raw_fd()
    }
}

#[cfg(any(debug_assertions, feature = "debug_logs"))]
impl DebugLog<InputAction> for History {
    fn log(&mut self, event: &InputAction) {
        self.debug_log
            .write_all(
                format!(
                    "[LOG::{:?} - {:?}] {{ values[{:?}] = '{:?}' }} - {:?} | temp = {:?}\r\n",
                    event,
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
                        Some(self.values[self.cursor - 1].clone())
                    },
                    self.values,
                    self.temp
                )
                .as_bytes(),
            )
            .unwrap();
    }

    fn dl_rfd(&self) -> i32 {
        self.debug_log.as_raw_fd()
    }
}
