use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;

use crate::console::winsize::winsize;
use crate::render_pipeline;
use crate::space::{area_conflicts, between, border_fit, Border, Padding};
use crate::themes::Style;

use super::Property;
use super::{ComponentTreeError, SpaceError};
use super::{Container, Term};

/// Text objects are direct children of the Container objects
/// and indirect children of the Term grand parent
#[derive(Debug, Default)]
pub struct Text {
    /// the layer of this Text inside its parent Container
    /// decide which Text takes render priority in case of conflict
    /// think of it like css z-index
    pub layer: u8,
    /// unique id
    pub id: [u8; 3],
    /// temporary value holder for use when scorrling history
    pub temp: Vec<Option<char>>,
    /// the value inside this Text object
    pub value: Vec<Option<char>>,
    /// history cursor current value
    pub hicu: usize,
    /// width
    pub w: u16,
    /// height
    pub h: u16,
    /// this Text's cursor x coordinate
    pub cx: u16,
    /// this Text's cursor y coordinate
    pub cy: u16,
    /// origin point x coordinate relative to the dimensions of the parent Container
    pub x0: u16,
    /// origin point y coordinate relative to the dimensions of the parent Container
    pub y0: u16,
    /// origin point x coordinate absolute value inside the Term
    pub ax0: u16,
    /// origin point y coordinate absolute value inside the Term
    pub ay0: u16,
    /// permits registry
    pub scopes: HashSet<&'static str>,
    /// border value
    pub border: Border,
    /// padding value
    pub padding: Padding,
    /// border style
    pub bstyle: String,
    /// value style
    pub vstyle: String,

    pub properties: HashMap<&'static str, Property>,
    pub attributes: HashSet<&'static str>,
}

// NOTE: Inputs can only have pair IDs
// while NonEdits can only have odd IDs
impl Text {
    /// creates a new Text objects
    /// takes most of Text's field values as arguments and returns a Text instance
    pub fn new(
        id: [u8; 3],
        x0: u16,
        y0: u16,
        ax0: u16,
        ay0: u16,
        w: u16,
        h: u16,
        value: &[Option<char>],
        border: Border,
        padding: Padding,
    ) -> Text {
        Text {
            id,
            w,
            h,
            temp: vec![],
            hicu: 0,
            x0,
            y0,
            ax0,
            ay0,
            properties: HashMap::new(),
            attributes: HashSet::new(),
            border,
            padding,
            value: {
                let mut v = Vec::with_capacity((w * h) as usize);
                v.resize((w * h) as usize, None);
                v.extend_from_slice(value);

                v
            },
            cx: 0,
            cy: 0,
            scopes: HashSet::new(),
            layer: 0,
            vstyle: "".to_string(),
            bstyle: "".to_string(),
        }
    }

    /// changes the value style of this container
    pub fn vstyle(&mut self, style: &Style) {
        self.vstyle = style.style();
    }

    /// changes the border style of this text
    pub fn bstyle(&mut self, style: &Style) {
        self.bstyle = style.style();
    }

    // pub fn with_layer(id: [u8; 3], layer: u8) -> Self {
    //     Text {
    //         layer,
    //         id,
    //         w: 37,
    //         h: 5,
    //         x0: 5,
    //         y0: 2,
    //     }
    // }

    /// returns the id of the parent container of this text
    pub fn parent(&self) -> [u8; 2] {
        [self.id[0], self.id[1]]
    }
}
