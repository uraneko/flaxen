<h1 align="center">
    ragout
</h1> 

[<img alt="github" src="https://img.shields.io/badge/github-uraneko.ragout-A5915F?style=for-the-badge&logo=github&labelColor=3a3a3a" height="25">](https://github.com/uraneko/ragout) 
[<img alt="crates.io" src="https://img.shields.io/crates/v/ragout.svg?style=for-the-badge&color=E40046&logo=rust&labelColor=3a3a3a" height="25">](https://crates.io/crates/ragout) 
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ragout-495c9f?style=for-the-badge&logo=docsdotrs&labelColor=3a3a3a" height="25">](https://docs.rs/ragout) 
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/uraneko/ragout/rust.yml?branch=main&style=for-the-badge&labelColor=3a3a3a" height="25">](https://github.com/uraneko/ragout/actions?query=branch%3Amain)
[<img alt="license" src="https://img.shields.io/github/license/uraneko/ragout?style=for-the-badge&labelColor=3a3a3a&color=ECD53F" height="25">](https://github.com/uraneko/ragout/blob/main/LICENSE)

<h3>
    A Terminal User Interface (TUI) Library
</h3>
 
ragout is a TUI library written in rust. Implementation wise, this crate sits between crossterm and ratatui. Check features to know what this can be used for. Check examples to know how to use. Check direction to know where the crate is heading.

## Support 
Works on Linux amd64 (if you have problems, open an issue describing your problem).

If the CI builds are passing then, the lib at least builds on Windows amd64 and Apple amd64/aarch64 systems (again, if you encounter a problem, open an issue describing what happened).

## Features
\- input: keyboard, mouse, window inputs 

✓ keyboard input: detect raw bytes keyboard input and decode it into keyboard input events

✓ mouse input: detect raw bytes mouse input and decode it into mouse input events (can be turned off)

✗ window input: detect window resize, focus and close events.

~ gamepad input: support for gamepad input, meant for ascii games.

\- themes: style components' text, backgrounds and borders.

✓ console utilities: support terminal modes; raw and cooked. Support for alternate screen and mouse input detection switch.

✗ fonts support: allow user to pick their font families for different texts. Support for double width/height lines.

✗ scroll: support vertical scrolling.

~ custom component shapes: support non rectangular components that take vertices instead of width and height.

✗ overlay: support for rendering components with overlay.

<br>
✗ not yet implemented 

~ not yet implemented, low priority.

\- work in progress

✓ implemented 

! implemented but buggy

### Installation

> [IMPORTANT]
> this crate does not have a working version (will be 0.4.0) yet.

```bash
# As this is a library crate, simply use
cargo add ragout 
```

## Examples
Refer to the examples <a href= "examples/README.md">README</a>.

## Version Mess
The versioning of the crate is a bit messy. Because, one, I'm new to this, and two, I didn't intend for this to be a real crate. 

At first, I was writing a cli tool and simply wanted the capability to move around and have already inputted value caching, which ended up being a bigger chore that I thought to implement, so I thought I'd share it. 

Which was fine, but then I started adding unneeded features and the design turned messy fast. Eventually, I decided to make a TUI crate from scratch (v0.4.0 and above).

> [!IMPORTANT] 
> This crate follows the [SemVer Spec](https://semver.org/) versioning scheme.
> Until the crate hits version 1.0.0, there are no rules, nonetheless, I'll try to make sense.

<br>

> [!CAUTION]
> You should not use versions < 0.4.0. Those are depricated, don't have the basic TUI features and have a messy design.

<br>

<hr height="1">

<br>

> [!IMPORTANT]
> Need help with testing different systems 

Licensed under <a href="LICENSE">MIT</a>
