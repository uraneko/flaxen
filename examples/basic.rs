use ragout::raw_mode::{cooked_mode, raw_mode};
use ragout::{decode_ki_kai, read_ki, Char, KbdEvent, Modifiers};

use std::io::Write;

fn main() {
    // kbd_read()

    // initialization
    let ts = raw_mode();

    let mut writer = std::io::stdout().lock();
    _ = writer.write(b"\x1b[?1049h\x1b[0;0f");
    _ = writer.flush();

    let mut reader = std::io::stdin().lock();

    let mut i = vec![];

    loop {
        read_ki(&mut reader, &mut i);
        print!("{:?}\r\n", &i);

        let ui = decode_ki_kai(i.drain(..).collect());

        print!("{:?}\r\n", &ui);
        _ = writer.flush();

        if let Ok(KbdEvent {
            char: Char::Char('c'),
            modifiers: Modifiers(2),
        }) = ui[0]
        {
            break;
        }
    }

    cooked_mode(&ts);
    _ = writer.write(b"\x1b[?1049l");
}
