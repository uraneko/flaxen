pub mod shin_sekai;

pub mod termbuf;

use termbuf::*;

struct Terminal {
    buffer: Vec<u8>,
    winsize: winsize,
}
