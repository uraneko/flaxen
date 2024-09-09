use events::*;
use ragout::*;
use ragout::{decode_ki_kai, read_ki, Char, KbdEvent, Modifiers};
use space::{Border, Padding};

use std::io::Write;

fn main() {
    let mut tree = object_tree::ObjectTree::new();
    let term = tree.term_ref_mut(0).unwrap();
    _ = term.container(
        &[0, 0],
        3,
        3,
        35,
        8,
        Border::Uniform('#'),
        Padding::Inner {
            top: 1,
            bottom: 1,
            right: 1,
            left: 1,
        },
    );
    _ = term.container(
        &[0, 1],
        56,
        15,
        35,
        8,
        Border::Uniform('+'),
        Padding::Inner {
            top: 1,
            bottom: 1,
            right: 1,
            left: 1,
        },
    );
    _ = term.input(
        &[0, 0, 0],
        "commander",
        1,
        1,
        23,
        2,
        Border::Uniform(':'),
        Padding::Inner {
            top: 0,
            bottom: 0,
            right: 1,
            left: 2,
        },
    );
    _ = term.input(
        &[0, 1, 0],
        "",
        3,
        3,
        25,
        1,
        Border::Uniform('?'),
        Padding::Inner {
            top: 0,
            bottom: 0,
            right: 1,
            left: 1,
        },
    );

    let style1 = themes::Style::new().bold().underline().txt(&[180, 59, 90]);

    let mut writer = std::io::stdout().lock();
    let mut reader = std::io::stdin().lock();
    // _ = writer.write(style1.style().as_bytes());

    let ts = raw_mode::raw_mode();
    enter_alternate_screen(&mut writer);

    term.clear(&mut writer);
    term.render(&mut writer);
    _ = term.make_active([0, 0, 0], &mut writer);
    _ = writer.flush();

    let mut i = vec![];

    let mut load = true;
    loop {
        ragout::frames(60);

        let ws = crate::winsize::from_ioctl();

        let (cols, rows) = (ws.cols(), ws.rows());
        if cols != term.w || rows != term.h {
            print!("\r\n\n\n\nabababababa");
            term.fire(crate::events::core::WindowResized::new(cols, rows));
        }

        read_ki(&mut reader, &mut i);
        let mut ui = decode_ki_kai(i.drain(..).collect());

        let ke = ui.remove(0).unwrap();

        term.fire((&ke, &mut writer, &ts));

        term.fire((&ke, &mut writer));

        if let Some(id) = term.active {
            let inobj = term.input_ref_mut(&id).unwrap();
            let key = inobj.name.clone();
            if load {
                term.load_input(&key);
                load = false;
            }
            let cache = term.cache.get(&key).unwrap_or(&vec![]).clone();
            // print!("\r\n\n\n\n\n\n{:?}", term.cache);
            let inobj = term.input_ref_mut(&id).unwrap();
            inobj.vstyle(&style1);
            let res = inobj.fire((&ke, &cache));
            // inobj.render_value(&mut writer);
            term.live_render(&mut writer);
            if !res.is_empty() {
                term.cache_input(&key, res);
                // print!("\r\n\n\n\n\n\n{:?}", term.cache);
            }
        }

        // term.reset_changed();

        _ = term.sync_cursor(&mut writer);
        _ = writer.flush();
    }

    // BUG: moving a Text Object cx and cy before switching to a different Text messes up the
    // rendering positions

    // raw_mode::cooked_mode(&ts);
    // leave_alternate_screen(&mut writer);
}
