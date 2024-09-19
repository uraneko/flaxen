//! ragout is a zero dependencies tui crate
#![allow(warnings)]
// #![deny(missing_docs)]
/// defines the 4 basic objects; ComponentTree, Term, Container and Text
pub mod components;
/// console utilities; winsize and termios (raw_mode)
pub mod console;
/// keyboard and mouse input detection and decoding
pub mod inputs;
/// rendering logic of the objects from data to the terminal display
pub mod render_pipeline;
/// space logic, such as area checks and border/padding definitions
pub mod space;
/// implements a Style type that abstracts the graphic rendition function of the vt100 video terminal
pub mod themes;

// TODO: object child position, vertical/horizontal center, start or end
// TODO: term switch event
// TODO: menu selection events
// TODO: input objects mevement events
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
