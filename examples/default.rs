use ragout::{init, run};

fn main() {
    // enter raw mode and initialize necessary variables
    let (mut sol, mut i, mut h, mut ui) = init("");

    'main: loop {
        // catch and handle user actions,
        // if input was submitted bind its value to input var
        let input = run(&mut i, &mut h, &mut sol, &mut ui);

        // reset user input to empty string
        if !input.is_empty() {
            // do some stuff with the user input
            ui.clear();
        }
    }
}
