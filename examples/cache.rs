use ragout::cache::*;
use ragout::object_tree::{ObjectTree, Term};

fn main() {
    let mut tree = ObjectTree::new();

    let term = tree.term_ref_mut(0).unwrap();

    let cache = vec![
        vec![Some('s'), None, Some('2')],
        vec![Some('r'), None, None, Some('d')],
        vec![Some('a'), Some('b'), None, Some('c')],
    ];

    cache
        .into_iter()
        .for_each(|c| term.cache_input("commander", c));

    println!("{:?}", term.cache);

    let cch = term.load_input("commander");

    println!("{:?}", cch);

    term.save_input("commander", cch);

    let cch = term.load_input("commander");

    println!("{:?}\n", cch);

    let si = serialize_input(&term.cache.get("commander").unwrap()[0]);
    let di = deserialize_input(&si);

    println!("{:?}\n{:?}", si, di);
}
