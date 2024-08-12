## ragout - terminal Raw Mode Input Handler

ragout is a library crate offering shell functionalities inside the terminal raw mode.
This crate is for cli tools that need functionalities beyond just the normally provided basic cli tool input mode (input characters, delete character behind cursor).

## Features
- **input movements**
    - move to start/end of input line
    - move to the next/prev item (basically a word) 
- **input deletion**
    - delete whole input line
    - delete all input to the right/left of cursor
- **history**
    - save input to input history on user submission (hits enter/return)
    - navigate through saved history entries with the up/down keys
- **exit** the program with CTRL-C (uses std::process::exit())

<br/><br/>

## Examples

#### default use case 

```rust

use ragout::{init, run};

fn main() {
    // enter raw mode and initialize necessary variables
    let (mut sol, mut i, mut h, mut ui) = init();
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

```

<br/><br/>

## License
Licensed under the <a href="LICENSE">MIT license</a>.

<br/><br/>

<b style="color: red">WARN:</b>
This crate is still experimental, if something breaks, or you want a feature, feel free to open an issue or make a pr.
