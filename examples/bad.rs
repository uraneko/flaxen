use ragout::commissioner::*;
use ragout::container::*;
use ragout::events::*;
use ragout::object_tree::*;
use ragout::*;

fn main() {
    let mut ot = ObjectTree::new();

    ot.fire(InitEvent::Container(&[0, 9]));
    ot.fire(InitEvent::NonEdit(&[0, 9, 5]));

    let mut writer = std::io::stdout().lock();

    let term = ot.term_ref_mut(0).unwrap();

    term.process();
    term.clear(&mut writer);
    term.render(&mut writer);

    // println!("{:?}", ot);
}
