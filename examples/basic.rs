use ragout::raw_mode::{cooked_mode, raw_mode};
use ragout::{decode_ki, read_ki};

use std::io::Write;

fn main() {
    // kbd_read()

    // initialization
    let ts = raw_mode();

    let mut writer = std::io::stdout().lock();
    _ = writer.write(b"\x1b[?1049h\r\n\x1b[0;0f");
    _ = writer.flush();

    let mut reader = std::io::stdin().lock();

    let mut i = vec![];

    loop {
        read_ki(&mut reader, &mut i);
        print!("{:?}\r\n", decode_ki(&i));
        _ = writer.flush();

        if i[0] == 3 && i.len() == 1 {
            break;
        }
    }

    cooked_mode(&ts);
    _ = writer.write(b"\x1b[?1049l");
}
