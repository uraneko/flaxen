use ragout::commissioner::*;
use ragout::container::*;
use ragout::events::*;
use ragout::*;

fn main() {
    let init = InitEvent::Text(true);

    let term = Term::<'s'>::new();

    let iti = term.fire(init).unwrap();
    let Err(iti) = iti else {
        unreachable!("wrong type initialized")
    };

    println!("{:?}", iti);
}
