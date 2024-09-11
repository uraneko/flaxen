#![allow(warnings)]
pub mod cache;
pub mod events;
pub mod input;
pub mod object_tree;
pub mod raw_mode;
pub mod render_pipeline;
pub mod space;
pub mod termbuf;
pub mod themes;

pub use input::keyboard::*;
pub use raw_mode::*;
pub use termbuf::*;

// TODO: text position, vertical/horizontal center, start or end
// TODO: term switch event
// TODO: menu selection events
// TODO: input objects mevement events
// FIXME: emojis take 2 cells instead of one, which easily ruins the rendering in many cases
// TODO: change objects to take vertices/edges instead of a width and height
// that way an object can have different shapes

pub fn frames(fps: u64) {
    let frames = 1000 / fps;

    std::thread::sleep(std::time::Duration::from_millis(frames))
}

use std::io::StdoutLock;
use std::io::Write;

pub fn leave_alternate_screen(writer: &mut StdoutLock) {
    _ = writer.write(b"\x1b[?1049l");
}

pub fn enter_alternate_screen(writer: &mut StdoutLock) {
    _ = writer.write(b"\x1b[?1049h\x1b[0;0f");
    _ = writer.flush();
}
