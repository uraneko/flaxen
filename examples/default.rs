// execute this shell command to run:
// $ cargo run --example default

use ragout::{init, run};

fn main() {
    // enter raw mode and initialize necessary variables
    // the string literal argument will be the value of the prompt
    let (mut sol, mut i, mut h, mut ui) = init("some prompt🢖 ", true);

    'main: loop {
        let input = run(&mut i, &mut h, &mut sol, &mut ui);
        if !input.is_empty() {
            // do some stuff with the user input
        }
    }
}
