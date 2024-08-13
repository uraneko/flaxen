## ragout - terminal Raw Mode Input Handler

ragout is a library crate offering shell functionalities inside the terminal raw mode.

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
    // the string literal argument will be the value of the prompt
    let (mut sol, mut i, mut h, mut ui) = init("");

    'main: loop {
        let input = run(&mut i, &mut h, &mut sol, &mut ui);
        if !input.is_empty() {
            // do some stuff with the user input
        }
    }
}

```

<br/><br/>

## License
Licensed under the <a href="LICENSE">MIT license</a>.

<br/><br/>

## Versioning 
Follows the [SemVer Spec](https://semver.org/).
Until the time arrives for the version to reach 1.0.0, the repo will adhere to the following rules for versions x.y.z:
- x is constant at 0.
- aside from a number of exceptions, changes incrementing y are accompanied by a milestone creation,
i.e., the first pr of a new milestone increments y.
- everything else increments z.

<br/><br/>

<b style="color: red">WARN:</b>
This crate is still unstable, if something breaks, or you want a feature, feel free to open an issue.
