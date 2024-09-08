#![allow(warnings)]
pub mod cache;
pub mod events;
pub mod kbd_decode;
pub mod mouse_input;
pub mod object_tree;
pub mod presets;
pub mod raw_mode;
pub mod render_pipeline;
pub mod space;
pub mod termbuf;
pub mod themes;

pub use kbd_decode::*;
pub use raw_mode::*;
pub use termbuf::*;

// TODO: object tree from vector of ids

// TODO:
// 1 => raw mode + alternate screen + winsize + term buffer of NUL... done
// 2 => kbd read + decode utf8... wip
// 3 => styled... wip... needs modifications
// 4 => event queue ... wip
// 5 containers... stalled
// 5a => inner input logic
// 5b => non editable text container logic (including prompt)
// 5c => popup container logic
// 6 => panes support

// TODO:
// 1 do space
// 2 add borders for the objects
// 3 do the builtin events;
//     input inner logic + output changes events + windowresize event + panes logic
// 4 do themes
//
// refactor space/rendering into a more robust approach

// now that raw mode, termsize reading, user input utf8 read/decoding, ids validity, space bounds, rendering and the events trait are done
// this big refactor should be over once themes and core builtin events are written
