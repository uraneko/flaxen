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
 
TODO!

## Support 
Works on Linux amd64 (if you have problems, open an issue describing your problem).

If the CI builds are passing then, the lib at least builds on Windows amd64 and Apple amd64/aarch64 systems (again, if you encounter a problem, open an issue describing what happened).

## Features
✓ inputs: keyboard, mouse and window(wip) input events.

✗ themes: style components' values and borders.

✓ raw/cooked terminal mode.

✓ optional mouse input.

✗ components overlay on/off

<br>
✓ the core logic of the feature is basically implemented

✗ the core logic of the feature is not implemented yet

## Examples

## Game
```shell
cargo run --example game
```

## Text Editor 
```shell
cargo run --example text_editor
```

## Direction
These features may be implemented next, if you want to know what's actively developed, check out the issues, prs and branches.

- component overlay support 
- better components api (e.g., Position::Center to center a component instead of having to pass parent.width / 2)
- gamepad input support
- better themes api

## Version Mess
The versioning of the crate is a bit messy. Because, one, I'm new to this, and two, I didn't intend for this to be a real crate. 

At first, I was writing a cli tool and simply wanted the capability to move around and have already inputted value caching, which ended up being a bigger chore that I thought to implement, so I thought I'd share it. 

Which was fine, but then I started adding unneeded features and the design turned messy fast. Eventually, I decided to make a TUI crate from scratch (v0.4.0 and above).

> [IMPORTANT] 
> This crate follows the [SemVer Spec](https://semver.org/).
> Until the crate hits version 1.0.0, there are no rules, nonetheless, I'll try to make sense.

<br>

> [!CAUTION]
> You should not use versions < 0.4.0. Those are depricated and have a messy design.

<br>

<hr height="1">

<br>

> [!IMPORTANT]
> Need help with testing different systems 
