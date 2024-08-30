#![allow(warnings)]
pub mod builders;
pub mod builtin_events;
pub mod commissioner;
pub mod container;
pub mod events;
pub mod history;
pub mod input;
pub mod kbd_decode;
pub mod presets;
pub mod raw_mode;
pub mod render_pipeline;
pub mod space_awareness;
pub mod styles;
pub mod termbuf;

pub use kbd_decode::*;
pub(crate) use raw_mode::*;
use termbuf::*;

use std::ops::Range;

pub mod object_tree;

use crate::styles::StyleStrategy;

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

use commissioner::Commissioner;

// TODO:
// 1 do space_awareness
// 2 add borders for the objects
// 3 do the builtin events;
//     input inner logic + output changes events + windowresize event + panes logic
// 4 do themes
//
// refactor space/rendering into a more robust approach
