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

### Basic usage
<br/><br/>

```sh
$ cargo run --example basic
```

<br/><br/>

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
<br/><br/>

```sh
$ cargo run --example macro --no-default-features --features custom_events
```

<br/><br/>

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
Until the time arrives for the version to reach 1.0.0, the repo will adhere to the following rules for versions x.y.z:
- x is constant at 0.
- aside from a number of exceptions, changes incrementing y are accompanied by a milestone creation,
i.e., the first pr of a new milestone increments y.
- everything else increments z. Consecutive small changes may be combined into a single incrementation of z.
- the above three rules are not always respected.

<br/><br/>

<b style="color: red">WARN:</b>
This crate is still unstable, if something breaks, or you want a feature, feel free to open an issue.
