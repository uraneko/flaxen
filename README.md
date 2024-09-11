<h1 align="center">
    ragout
</h1> 

[<img   alt="github" src="https://img.shields.io/badge/github-uraneko.ragout-A5915F?style=for-the-badge&logo=github&labelColor=3a3a3a" height="25">](https://github.com/uraneko/ragout) 
[<img alt="crates.io" src="https://img.shields.io/crates/v/ragout.svg?style=for-the-badge&color=E40046&logo=rust&labelColor=3a3a3a" height="25">](https://crates.io/crates/ragout) 
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ragout-495c9f?style=for-the-badge&logo=docsdotrs&labelColor=3a3a3a" height="25">](https://docs.rs/ragout) 
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/uraneko/ragout/rust.yml?branch=main&style=for-the-badge&labelColor=3a3a3a" height="25">](https://github.com/uraneko/ragout/actions?query=branch%3Amain)
[<img alt="license" src="https://img.shields.io/github/license/uraneko/ragout?style=for-the-badge&labelColor=3a3a3a&color=ECD53F" height="25">](https://github.com/uraneko/ragout/blob/main/LICENSE)

<h3 align="center">
    Terminal User Interface (TUI) Library
</h3>
 
This crate aims to facilitate the creation of terminal programs, such as cli tools, ascii/unicode games or full fledged tui programs.
Features wise, it probably sits between ratatui and crossterm (I have not used either).
Notable Characteristics: 
- zero dependencies
- lightweight 
- mid to low level abstraction to various terminal utilities including keyboard input reading or object rendering into the terminal display
- implement your own events 

## Support 
I'm developing on Hyprland on arch linux, so the hundreds of millions of wayland loving arch/hyprland users can rest assured that their machines are most likely supported. The majority of linux users should mostly have no problem using this lib as well. As for niche systems like windows or apple, I can try to test on a windows vm. But serious support of both systems would need the help of a good samaritan like yourself.

## Features

## Examples

## What is this and Why?

<details>
    <summary>It all started when I began writing a simple cli tool</summary>

I was making cli tool, then I

🠊 needed more user input maneuverability/functionality 

🠊 did some reseach and learnt of terminal raw mode 

🠊 imported crossterm's raw_mode and keyboard event reading functionalities 

🠊 had to implement my own user input movement + insertion/deletion logic (but hey, now I could implement whatever I wanted) 

🠊 could return to making the cli tool that now has shiny user input, now that the new advanced user input module was ready

🠊 thought that it was a pain how I had to do all of that because I just wanted to move to the right and left while writing input in terminal and couldn't find a small crate that does that

🠊 made the crossterm dependent raw mode based user input logic handler module into an independent crate and published it

🠊 planned to refactor this new crate because it has an unsatisfactory desgin

🠊 instead of refactoring the newly published crate and calling it a day, went on adding new features, I even made a 'not really the right situation for it'  proc-macro, which massively inflated the issues of the poor design

🠊 decided to properly redo the crate from the ground up while eliminating all dependencies 

🠊 redid the crate from the ground up

🠊 ended up with v0.4.0 of this crate: a TUI library
</details>

## What Next
For now, there is some basic functionality that still needs to be implemented (contributions are welcome), then I'll see what features crates such as ratatui and crossterm provide and add those that I deem suitabe for the scope and direction of ragout.

<!-- ## License -->
<!-- Licensed under the <a href="LICENSE">MIT license</a>. -->
<!-- > [!IMPORTANT]  -->
<!-- > Contributions / Copyright -->

## Versioning
Follows the [SemVer Spec](https://semver.org/).
Until the crate hits version 1.0.0, there are no rules, nonetheless, I'll try to make sense.

> [!WARNING]
> You should not use versions < 0.4.0. They are poorly designed and more of a draft of the crate.

<hr>

> [!IMPORTANT]
> Contributions are very welcome, especially for testing support on different systems.

> [!NOTE]
> As this is my first open source project and I'm new to github in general, any feedback or criticism would be really appreciated.

