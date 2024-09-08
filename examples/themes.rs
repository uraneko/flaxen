use events::*;
use ragout::*;
use ragout::{decode_ki_kai, read_ki, Char, KbdEvent, Modifiers};
use space::{Border, Padding};

use std::io::Write;

fn main() {
    let mut tree = object_tree::ObjectTree::new();
    let mut term = tree.term_ref_mut(0).unwrap();
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
    _ = writer.write(b"\x1b[?1049h\x1b[0;0f");
    _ = writer.flush();

    term.clear(&mut writer);
    term.render(&mut writer);
    _ = term.make_active([0, 0, 0], &mut writer);
    _ = writer.flush();

    let mut i = vec![];

    loop {
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

        if let Some(id) = term.fire(&ke) {
            _ = term.make_active(id, &mut writer);
        }

        // this should have been: term's active object render
        // FIXME: add term active object and render it in the loop
        if let Some(id) = term.active {
            let cache = term.cache.get("input").unwrap_or(&vec![]).clone();
            let input_object = term.input_ref_mut(&id).unwrap();
            input_object.vstyle(&style1);
            let res = input_object.fire((&ke, &cache));
            // input_object.render_value(&mut writer);
            term.live_render(&mut writer);
            if !res.is_empty() {
                term.cache_input(res);
            }
        }

        // term.reset_changed();

        _ = term.sync_cursor(&mut writer);
        _ = writer.flush();
    }

    // BUG: moving a Text Object cx and cy before switching to a different Text messes up the
    // rendering positions
    // weird bug

    raw_mode::cooked_mode(&ts);
    _ = writer.write(b"\x1b[?1049l");
}

// TODO: most of the ways i calculate the text position for use in rendering are broken
// text objects can just have their absolute positions as fields upon creation
// TODO: parent method for both container and text objects that  returns the parent of the object
// it is called on
// TODO: need a graph/map like data structure that keeps track of what objects i can switch to +
// another that keeps track of what objects need to be re-rendered from an event loop iteration to
// anotherm ie. what objects have been interacted with
