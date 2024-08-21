use ragout::{kbd_read, Terminal};

fn main() {
    let mut term = Terminal::new();

    kbd_read();
}
