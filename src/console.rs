/// termios c ffi, raw mode utilities
pub mod raw_mode;
/// winsize c ffi, use for getting the terminal window widtn and height
pub mod winsize;

use std::io::StdoutLock;
use std::io::Write;

/// exits the terminal alternate screen back to the original screen
pub fn leave_alternate_screen(writer: &mut StdoutLock) {
    _ = writer.write(b"\x1b[?1049l");
}

/// move to the terminal alternate screen from the defaut one
pub fn enter_alternate_screen(writer: &mut StdoutLock) {
    _ = writer.write(b"\x1b[?1049h\x1b[0;0f");
    _ = writer.flush();
}
