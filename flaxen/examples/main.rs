use lib::{init, run};

fn main() {
    let (mut sol, mut i, mut h, mut ui) = init();
    'main: loop {
        let tokens = run(&mut i, &mut h, &mut sol, &mut ui);

        // eprintln!("{:?}", tokens);
    }
}
