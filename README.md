## ragout - terminal Raw Mode Input Handler

<!-- ![CRATES.IO](https://github.com/uraneko/ragout/actions/workflows/main.yml/crates-io.svg?branch=BRANCH-NAME&event=push) -->
<!---->
<!-- ![DOCS.RS](https://github.com/uraneko/ragout/actions/workflows/main.yml/crates-io.svg?branch=BRANCH-NAME&event=push) -->
<!---->
<!-- ![GITHUB](https://github.com/uraneko/ragout/actions/workflows/main.yml/crates-io.svg?branch=BRANCH-NAME&event=push) -->
<!---->
![BUILD](https://github.com/uraneko/ragout/actions/workflows/rust.yml/build.svg?branch=BRANCH-NAME&event=push)

ragout is a library crate offering shell functionalities inside the terminal raw mode.
It aims to be lightweight and tries not to get in the way by offering a limited api in a small sized crate with minimal dependencies.

This lib is for projects that want a little more functionality than the typical cli tool has, but don't want to use something fully equipped like crossterm.

## Support 
Supports only Linux, plans for supporting Windows and maybe Apple are there, but are currently not a priority.

[!NOTE]
This crate is currently undergoing heavy refactoring in the shin_sekai branch.

<br/>

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

### Basic usage

```sh
$ cargo run --example basic
```

```rust
use ragout::{init, run};

fn main() {
    // enter raw mode and initialize necessary variables
    // the string literal argument will be the value of the prompt
    let (mut sol, mut i, mut h, mut ui) = init("some prompt 🐱 ", true);

    'main: loop {
        let input = run(&mut i, &mut h, &mut sol, &mut ui);
        if !input.is_empty() {
            // do some stuff with the user input
        }
    }
}
```

<br/><br/>

### Using the macro 

```sh
$ cargo run --example macro --no-default-features --features custom_events
```

```rust
use ragout::ragout_custom_events;

ragout_custom_events! {
    KeyCode::F(5), 0x0, TestF(u8),
    || {
        let date = std::process::Command::new("date")
            .output()
            .unwrap()
            .stdout.into_iter()
            .map(|u| u as char)
            .collect::<String>()
            .replacen("\"", "", 2);


        self.overwrite_prompt(date
            .trim_end_matches('\n'));
        self.write_prompt(sol);
        // TODO: sol.write input, should be called from inside input.write_prompt() right before
        // sol.flush() at the end
    };
    KeyCode::Esc, 0x0, TestPrintScreen,
    || {
        // requires that the grim cli tool (or something similar, replace as needed) is installed
        let cmd = std::process::Command::new("grim").arg("target/screenshot.png").output().unwrap();

        let inst = std::time::Instant::now();

        let temp = self.prompt.drain(..).collect::<String>();
        self.overwrite_prompt("saved screenshot to target/screenshot.png> ");
        self.write_prompt(sol);

        let notify =  std::thread::spawn(move || loop {
                if inst.elapsed() > std::time::Duration::from_secs(3) {

                    break true;
                }
        });

        let notify = notify.join().unwrap();
        if notify {
            self.overwrite_prompt(&temp);
            self.write_prompt(sol);
        }

    };
}

fn main() {
    let (mut sol, mut i, mut h, mut ui) = init("some prompt 🐭 ", true);

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
~~ Until the time arrives for the version to reach 1.0.0, the repo will adhere to the following rules for versions x.y.z:
- x is constant at 0.
- aside from a number of exceptions, changes incrementing y are accompanied by a milestone creation,
i.e., the first pr of a new milestone increments y.
- everything else increments z. Consecutive small changes may be combined into a single incrementation of z.
- the above three rules are not always respected. ~~

Until the crate hits version 1.0.0, there are no rules

<br/><br/>

<b style="color: red">WARN:</b>
This crate is still unstable, if something breaks, or you want a feature, feel free to open an issue.
