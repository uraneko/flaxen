use ragout::events::*;
use ragout::object_tree::*;
use ragout::space::*;

fn main() {
    let mut ot = ObjectTree::new();

    let term = ot.term_ref_mut(0).unwrap();

    _ = term.container(&[0, 9], 3, 2, 54, 16, Border::None, Padding::None);
    _ = term.input(
        &[0, 9, 2],
        6,  // x0
        0,  // y0
        30, // w
        1,  // h
        Border::None,
        Padding::None,
    );

    _ = term.nonedit(
        &[0, 9, 1],
        27, // x0
        7,  // y0
        // BUG: setting width to 6 would break the example
        // because the value writing for loop will try to access value[5] which crashes since
        // value.len = 5 and so looping on 0..width stops after idx 4 of value with width = 5
        12, // w
        2,  // h
        &[
            Some('w'),
            Some('o'),
            Some('r'),
            Some('l'),
            Some('d'),
            Some('!'),
        ],
        false,
        Border::None,
        Padding::None,
    );

    _ = term.container(&[0, 2], 86, 19, 53, 14, Border::None, Padding::None);
    _ = term.nonedit(
        &[0, 2, 5],
        6,  // x0
        1,  // y0
        15, // w
        1,  // h
        // border *
        // padding outer: 1 1 1 1
        &[Some('h'), Some('e'), Some('l'), Some('l'), Some('o')],
        false,
        Border::None,
        Padding::None,
    );

    _ = term.nonedit(
        &[0, 2, 1],
        27, // x0
        6,  // y0
        // BUG: setting width to 6 would break the example
        // because the value writing for loop will try to access value[5] which crashes since
        // value.len = 5 and so looping on 0..width stops after idx 4 of value with width = 5
        4, // w
        2, // h
        // border *
        // padding outer: 1 1 1 1
        &[
            Some('w'),
            Some('o'),
            Some('r'),
            Some('l'),
            Some('d'),
            Some('!'),
        ],
        false,
        Border::None,
        Padding::None,
    );

    let mut writer = std::io::stdout().lock();

    let term = ot.term_ref_mut(0).unwrap();

    term.clear(&mut writer);
    term.render(&mut writer);
}
