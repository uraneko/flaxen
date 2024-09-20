/// keyboard raw input listening and decoding into human readable keyboard input events
pub mod keyboard;
/// mouse/touchpad raw input listening and decoding into human readable keyboard input events
pub mod mouse;
/// window user input events, such as a resize or a focus change
pub mod window;

use keyboard::{decode_ki, decode_ki_kai, Char, KbdEvent, PasteEvent};
use mouse::{decode_mi, MouseEvent};
use window::WindowEvent;

use std::io::BufRead;
use std::io::StdinLock;
use std::time::SystemTime;

use crate::components::Term;
use crate::console::winsize::winsize;

/// reads the keyboard input event bytes
pub fn read<'a>(reader: &'a mut StdinLock, buffer: &'a mut Vec<u8>) -> &'a mut Vec<u8> {
    // TODO: non blocking reads
    buffer.clear();

    let buf = reader.fill_buf().unwrap();
    buffer.extend_from_slice(buf);

    let n = buf.len();
    reader.consume(n);

    buffer
}

/// resolves the read input bytes to an InputEvent struct instance
pub fn event(bytes: &[u8], ws: &mut winsize) -> InputEvent {
    // FIXME: this only gets triggered on this blocking event fn
    // it needs to be triggered on the actual resize event
    if ws.resized() {
        return InputEvent {
            time: SystemTime::now(),
            event: UserInputEvent::WindowEvent(WindowEvent::WindowResized),
        };
    } else if bytes.len() % 6 == 0 && bytes[..3] == [27, 91, 77] {
        // mouse
        return InputEvent {
            time: SystemTime::now(),
            event: UserInputEvent::MouseEvent(decode_mi(bytes).remove(0)),
        };
    } else if bytes.len() < 9 {
        // BUG: 'لا' arabic char breaks the decode_ki function since it's 2 unicode chars combined char
        // i could use decode_ki_kai and take the first char only, but that breaks the combined
        // char
        // TODO: implement unicode combined chars support
        // keyboard
        return InputEvent {
            time: SystemTime::now(),
            event: UserInputEvent::KbdEvent(decode_ki(bytes).unwrap()),
            // event: UserInputEvent::KbdEvent(decode_ki_kai(bytes.to_vec()).remove(0).unwrap()),
        };
    } else {
        // paste
        return InputEvent {
            event: UserInputEvent::PasteEvent(PasteEvent(
                decode_ki_kai(bytes.to_vec())
                    .into_iter()
                    .filter(|r| r.is_ok())
                    .map(|r| r.unwrap())
                    .filter(|c| c.is_char())
                    .map(|c| {
                        let Char::Char(ch) = c.char else {
                            unreachable!()
                        };
                        ch
                    })
                    .collect::<String>(),
            )),
            time: SystemTime::now(),
        };
    }
}

#[derive(Debug)]
pub enum UserInputEvent {
    WindowEvent(WindowEvent),
    KbdEvent(KbdEvent),
    MouseEvent(MouseEvent),
    PasteEvent(PasteEvent),
}

#[derive(Debug)]
pub struct InputEvent {
    pub event: UserInputEvent,
    pub time: SystemTime,
}
