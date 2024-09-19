use std::io::BufRead;

use std::io::Read;
use std::io::Write;
// from /usr/include/termios.h
extern "C" {
    fn tcgetattr(__fd: i32, __termios_p: *mut termios) -> i32;
    fn tcsetattr(__fd: i32, __optional_actions: i32, __termios_p: *const termios) -> i32;
}

/// ffi bindings for the c struct found here 'from /usr/include/asm-generic/termbits.h'
/// termios can be used to manipulate terminal flags
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct termios {
    c_iflag: tcflag_t,
    c_oflag: tcflag_t,
    c_cflag: tcflag_t,
    c_lflag: tcflag_t,
    c_line: cc_t,
    c_cc: cc_t,
}

// from /usr/include/libr/sflib/common/sftypes.h
type cc_t = [u8; 8];
type tcflag_t = u32;

// from /usr/include/libr/sflib/common/sftypes.h
const TCSANOW: i32 = 0;
const TCSADRAIN: i32 = 1;
const TCSAFLUSH: i32 = 2;

// from /usr/include/unistd.h
const STDIN_FILENO: i32 = 0;
const STDOUT_FILENO: i32 = 1;
const STDERR_FILENO: i32 = 2;

// It is recommended to read [this]('https://www.gnu.org/software/libc/manual/html_node/Input-Modes.html') before using the tcsetattr fn directly if you don't know what the various fileds of termios are and what their flags do
// Otherwise, if you intend to only use tcgetattr or the abstract enable/disable_raw/rare_mode fns then no reading is required
// flags definitions can be found in these header files: '/usr/include/bits/termios-c_*.h'
// Recommended to read [this]('https://smnd.sk/anino/programming/c/unix_examples/raw.html') if you want to understand the reasoning behind the particular flag configuration of raw_mode

// # [important]("https://www.gnu.org/software/libc/manual/html_node/Terminal-Modes.html")
// # lflag bits
// turning this flag off means input won't be displayed on the terminal anymore
const ECHO: u32 = 10;
// turning this flag off disables the INTR, QUIT and SUSP signals
// basically, when this is off, stuff like ctrl-c (SIGINT) won't work
// turning this bit off renders c_cc's special characters off
const ISIG: u32 = 1;
// canonical input mode gives meaning to escape sequences,
// without it 'Backspace' would not delete the char behind cursor and 'Enter' will not submit input
const ICANON: u32 = 2;
//
const IEXTEN: u32 = 100_000;

// # iflag bits
// disabling this disables signal interrupt on break, we don't want it in raw mode
const BRKINT: u32 = 2;
// when this is enabled input is checked for parity,
// this bit is a pair with cflag's PARENB bit
const INPCK: u32 = 20;
// this strips the 8th bit off an input ascii char / byte
const ISTRIP: u32 = 40;
// maps LF to CR
const INLCR: u32 = 100;
// this bit maps CR to NL, meaning that it automatically prepends a cr (\r) on lf (\n)
const ICRNL: u32 = 400;
// machine ouput start/stop control
// we'll turn this off, but keep its input counterpart
const IXON: u32 = 2000;

// # oflag bits
// this bit enables output post processing, processes output before displaying it so that it is
// rendered correctly on the terminal
const OPOST: u32 = 1;

// # cflag bits
// specifies that a byte will be 8 bits
const CS8: u32 = 60;
// when this is enabled a parity bit is added to output values
const PARENB: u32 = 400;

/// enables raw mode through disabling the relevant terminal flags - mainly ECHO and CANONICAL mode
/// returns the original terminal flags in a termios instance for use when disabling raw mode
/// read the source code to know more about what gets disabled
pub fn raw_mode() -> termios {
    unsafe {
        let mut original: termios = std::mem::zeroed();
        let _res = tcgetattr(STDIN_FILENO, &mut original);
        let mut raw = original.clone();
        raw.c_lflag &= !(ISIG | ICANON | ECHO | IEXTEN);
        raw.c_iflag &= !(BRKINT | INPCK | ISTRIP | INLCR | ICRNL | IXON);
        raw.c_oflag &= !OPOST;
        raw.c_cflag &= !PARENB;
        raw.c_cflag |= CS8;

        let _res = tcsetattr(STDIN_FILENO, TCSANOW, &raw);

        original
    }
}

/// disables raw mode to original flags configuration
/// takes the original flags from the 'original' termios instance returned from enable_raw_mode
pub fn cooked_mode(original: &termios) -> i32 {
    unsafe { tcsetattr(STDIN_FILENO, TCSANOW, original) }
}

/// enables rare mode
/// a terminal mode that is between cooked and raw
/// in terms of features and functionality
fn rare_mode() {}
