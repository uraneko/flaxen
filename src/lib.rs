//! ragout is a zero dependencies tui crate
#![allow(warnings)]
// #![deny(missing_docs)]
/// defines the 4 basic objects; ComponentTree, Term, Container and Text
pub mod components;
/// console utilities; winsize and termios (raw_mode)
pub mod console;
/// keyboard and mouse input detection and decoding
pub mod inputs;
pub mod overlay;
/// rendering logic of the objects from data to the terminal display
pub mod render_pipeline;
/// space logic, such as area checks and border/padding definitions
pub mod space;
/// implements a Style type that abstracts the graphic rendition function of the vt100 video terminal
pub mod themes;

pub mod layout;

// INFO: [very useful](https://vt100.net/docs/vt510-rm/contents.html)

// TODO: scrolling probably use \x1b[y0;y1r
// for many components on the same height
// turn scrolling on and off based on focused component
// TODO: double width/height lines
// TODO: font size and family changes
// TODO: object child position, vertical/horizontal center, start or end
// TODO: term switch event // for extended
// TODO: menu selection events // for extended
// TODO: change objects to take vertices/edges instead of a width and height that way an object can have different shapes
// TODO: layers and overlay
// BUG: some unicode characters take more space than one cell
// emojis take 2 cells instead of one, which easily ruins the rendering in many cases

/// Decides how many times the event loop will run in 1 second.
///
/// This function is just a wrapper around a sleep call to the main thread
///
/// if you pass an fps value of 60, that means you event loop will run 60 times per second
///
/// it is recommended to use this function, as letting the loop run withut a limiter would
/// overwork the cpu even with an empty loop, since the latter would run the loop as fast as it can
///
/// # Errors
///
///
/// # Examples
/// ```
/// fn main() {
///     loop {
///         frames(60);
///         println!("I get printed 60 times / second");
///     }
/// }
/// ```
pub fn frames(fps: u64) {
    let frames = 1000 / fps;

    std::thread::sleep(std::time::Duration::from_millis(frames))
}

// STEPS:
// 1 define your component tree
//      1.1 define your layouts
//      1.2 define your sizes, positions, borders, paddings
//      1.3 define your fonts, styles, themes
//      1.4 define you output events and components behavior
//      1.5 init the tree
// 2 setup your environment
//      2.1 call raw mode
//      2.2 call alternate screen
//      2.3 optionally turn on raw mouse inputs
//      2.4 call events to read input events
//      2.5 init you terminal reader and writer
// 3 write your event loop
//      3.1 write your event loop logic
//      3.2 don't forget to clean up before leaving the program
//          3.2.1 call cooked mode
//          3.2.2 leave alternate screen
//          3.3.3 disable raw mouse inputs in case you enabled it
