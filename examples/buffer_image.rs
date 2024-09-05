use ragout::object_tree::Term;

fn main() {
    let term = Term::new(5);

    println!("{:?}", term);

    term.print_buf();
}
