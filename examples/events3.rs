use ragout::commissioner::*;
use ragout::container::*;
use ragout::events::*;
use ragout::object_tree::*;
use ragout::raw_mode::{cooked_mode, raw_mode};
use ragout::*;
use ragout::{decode_ki_kai, read_ki, Char, KbdEvent, Modifiers};

use std::io::Write;

fn main() {
    // go into raw mode + some prep
    let ts = raw_mode();
    let mut writer = std::io::stdout().lock();
    _ = writer.write(b"\x1b[?1049h\x1b[0;0f");
    _ = writer.flush();

    // init the object tree and some children
    let mut ot = ObjectTree::new();
    let term = ot.term_ref_mut(0).unwrap();

    term.fire(InitEvent::Container(&[0, 9], 3, 2, 54, 16));
    term.fire(InitEvent::Input(
        &[0, 9, 2],
        6,  // x0
        0,  // y0
        35, // w
        6,  // h
            // border *
            // padding outer: 1 1 1 1
            // &['h', 'e', 'l', 'l', 'o'],
    ));

    let pos = term.locate_text(&[0, 9, 2]).unwrap();

    let res = term.make_active([0, 9, 2], &mut writer);
    if let Err(_) = res {
        std::process::exit(0)
    }

    let mut reader = std::io::stdin().lock();
    let mut i = vec![];

    let fps = 60;
    let refresh = 1000 / fps;

    term.clear(&mut writer);
    term.render(&mut writer);
    _ = writer.flush();
    loop {
        // get keyboard event from input
        read_ki(&mut reader, &mut i);
        let mut ui = decode_ki_kai(i.drain(..).collect());

        // if ctrl-c then quit the loop
        if let Ok(KbdEvent {
            char: Char::Char('c'),
            modifiers: Modifiers(2),
        }) = ui[0]
        {
            break;
        }

        let ke = ui.remove(0).unwrap();

        let input_object = term.input_ref_mut(&[0, 9, 2]).unwrap();
        _ = input_object.fire((&ke, &mut writer));

        input_object.render_value(&mut writer, &pos);
        input_object.render_border(&mut writer, &pos);
        // refresh the display with new changes
        // print!("\r\n\n{:?}", input_object.value);

        _ = term.sync_cursor(&mut writer);

        _ = writer.flush();

        // refresh rate
        std::thread::sleep(std::time::Duration::from_millis(refresh));
    }

    cooked_mode(&ts);
    _ = writer.write(b"\x1b[?1049l");
}
