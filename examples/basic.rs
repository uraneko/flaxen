use ragout::kbd_decode::read;
use ragout::raw_mode::{cooked_mode, raw_mode};
use ragout::{kbd_read, Terminal};

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::StdinLock;
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::io::FromRawFd;

fn main() {
    // kbd_read()

    // initialization
    let ts = raw_mode();

    let mut sol = std::io::stdout().lock();

    let mut sil = std::io::stdin().lock();

    let mut i = vec![];

    loop {
        read(&mut sil, &mut i);
        print!("{:?}\r\n", &i);
        _ = sol.flush();

        if i[0] == 3 && i.len() == 1 {
            break;
        }
    }

    cooked_mode(&ts);
}
