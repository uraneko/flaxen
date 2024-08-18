// execute this shell command to run:
// $ cargo run --example styled

// INFO: Although this example showcases how to use the styled feature - responsible for
// coloring/text effects - the feature is still uncomplete.
// If you run the example below, you will see that right now the effects are applied universally, which is not very useful

use ragout::styled::{Styled, Stylize};
use ragout::{init, run};

fn main() {
    let (mut sol, mut i, mut h, mut ui) = init("some prompt 🐱 ", true);
    let mut s = Styled::new();
    s.bold();
    s.underline();
    s.txt(&[123, 142, 165]);
    s.bkg(&[1, 76, 41]);

    let mut stylize = s.styled();
    stylize.apply(&mut sol);

    'main: loop {
        let input = run(&mut i, &mut h, &mut sol, &mut ui);
        if !input.is_empty() {
            match input.trim() {
                "r" => {
                    s.reset();
                    s.calibrate(&mut stylize);
                    stylize.apply(&mut sol);
                }
                _ => (),
            }
        }
    }
}
