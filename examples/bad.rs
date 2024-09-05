use ragout::commissioner::*;
use ragout::container::*;
use ragout::events::*;
use ragout::object_tree::*;
use ragout::*;

fn main() {
    let mut ot = ObjectTree::new();

    let mut term = ot.term_ref_mut(0).unwrap();

    term.fire(InitEvent::Container(&[0, 9], 3, 2, 54, 16));
    term.fire(InitEvent::Input(
        &[0, 9, 2],
        6,  // x0
        0,  // y0
        30, // w
        1,  // h
            // border *
            // padding outer: 1 1 1 1
            // &['h', 'e', 'l', 'l', 'o'],
    ));

    term.fire(InitEvent::NonEdit(
        &[0, 9, 1],
        27, // x0
        7,  // y0
        // BUG: setting width to 6 would break the example
        // because the value writing for loop will try to access value[5] which crashes since
        // value.len = 5 and so looping on 0..width stops after idx 4 of value with width = 5
        12, // w
        2,  // h
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
    ));
    // TODO: if a value is given for text at initialization, it needs to be checked for bounds
    // validity,

    // println!("{:#?}", ot.container_ref(&[0, 9]));

    // ot.container_ref_mut(&[0, 9]).unwrap().items.push(Text {
    //     value: vec!['s', 'e', 'c', 'o', 'n', 'd', ' ', 'i', 't', 'e', 'm'],
    //     x0: 3,
    //     y0: 3,
    //     ..Default::default()
    // });

    // println!("{:#?}", ot);

    term.fire(InitEvent::Container(&[0, 2], 86, 19, 53, 14));
    term.fire(InitEvent::NonEdit(
        &[0, 2, 5],
        6,  // x0
        1,  // y0
        15, // w
        1,  // h
        // border *
        // padding outer: 1 1 1 1
        &[Some('h'), Some('e'), Some('l'), Some('l'), Some('o')],
    ));

    term.fire(InitEvent::NonEdit(
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
    ));

    let mut writer = std::io::stdout().lock();

    let term = ot.term_ref_mut(0).unwrap();

    term.clear(&mut writer);
    term.render(&mut writer);
}
