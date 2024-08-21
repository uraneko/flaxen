// use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
//
// fn main() {
//     unsafe {
//         let mut ws: winsize = std::mem::zeroed();
//         let res = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);
//         print!("{:?}", ws);
//     }
// }

// from /usr/include/sys/ioctl.h
#[link(name = "c")]
extern "C" {
    fn ioctl(__fd: i32, __request: u64, ...) -> i32;
}

// from /usr/include/asm-generic/termios.h
#[derive(Debug)]
#[repr(C)]
pub(crate) struct winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

// from /usr/include/unistd.h
const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;
const STDERR_FILENO: i32 = 2;

// from /usr/include/asm-generic/ioctl.h
const TIOCGWINSZ: u64 = 0x5413;
const TIOCSWINSZ: u64 = 0x5414;

fn main() {
    unsafe {
        let mut ws: winsize = std::mem::zeroed();
        let _res = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);
        // WARN: tried giving TIOCSWINSZ nonsense values, it executed safely witha return value of
        // [0], it also broke the terminal instance i was on
        println!("im the res = {:?}", ws);
    }
}

/// CPR – Cursor Position Report – VT100 to Host
/// ESC [ Pn ; Pn R 	default value: 1
///
/// The CPR sequence reports the active position by means of the parameters. This sequence has two parameter values, the first specifying the line and the second specifying the column. The default condition with no parameters present, or parameters of 0, is equivalent to a cursor at home position.
///
/// The numbering of lines depends on the state of the Origin Mode (DECOM).
///
/// This control sequence is solicited by a device status report (DSR) sent from the host.
/// CUB – Cursor Backward – Host to VT100 and VT100 to Host
/// ESC [ Pn D 	default value: 1
///
/// The CUB sequence moves the active position to the left. The distance moved is determined by the parameter. If the parameter value is zero or one, the active position is moved one position to the left. If the parameter value is n, the active position is moved n positions to the left. If an attempt is made to move the cursor to the left of the left margin, the cursor stops at the left margin. Editor Function
/// CUD – Cursor Down – Host to VT100 and VT100 to Host
/// ESC [ Pn B 	default value: 1
///
/// The CUD sequence moves the active position downward without altering the column position. The number of lines moved is determined by the parameter. If the parameter value is zero or one, the active position is moved one line downward. If the parameter value is n, the active position is moved n lines downward. In an attempt is made to move the cursor below the bottom margin, the cursor stops at the bottom margin. Editor Function
/// CUF – Cursor Forward – Host to VT100 and VT100 to Host
/// ESC [ Pn C 	default value: 1
///
/// The CUF sequence moves the active position to the right. The distance moved is determined by the parameter. A parameter value of zero or one moves the active position one position to the right. A parameter value of n moves the active position n positions to the right. If an attempt is made to move the cursor to the right of the right margin, the cursor stops at the right margin. Editor Function
/// CUP – Cursor Position
/// ESC [ Pn ; Pn H 	default value: 1
///
/// The CUP sequence moves the active position to the position specified by the parameters. This sequence has two parameter values, the first specifying the line position and the second specifying the column position. A parameter value of zero or one for the first or second parameter moves the active position to the first line or column in the display, respectively. The default condition with no parameters present is equivalent to a cursor to home action. In the VT100, this control behaves identically with its format effector counterpart, HVP. Editor Function
///
/// The numbering of lines depends on the state of the Origin Mode (DECOM).
/// CUU – Cursor Up – Host to VT100 and VT100 to Host
/// ESC [ Pn A 	default value: 1
///
/// Moves the active position upward without altering the column position. The number of lines moved is determined by the parameter. A parameter value of zero or one moves the active position one line upward. A parameter value of n moves the active position n lines upward. If an attempt is made to move the cursor above the top margin, the cursor stops at the top margin. Editor Function
fn box_drawing() {}
