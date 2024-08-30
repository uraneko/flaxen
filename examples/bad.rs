use ragout::commissioner::*;
use ragout::container::*;
use ragout::events::*;
use ragout::object_tree::*;
use ragout::*;

fn main() {
    let mut ot = ObjectTree::new();

    ot.fire(InitEvent::Container(&[0, 9], 3, 3, 21, 6));
    ot.fire(InitEvent::NonEdit(
        &[0, 9, 5],
        12, // x0
        1,  // y0
        5,  // w
        1,  // h
        // border *
        // padding outer: 1 1 1 1
        &['h', 'e', 'l', 'l', 'o'],
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

    let mut writer = std::io::stdout().lock();

    let term = ot.term_ref_mut(0).unwrap();

    // term.clear(&mut writer);
    term.render(&mut writer);
}
