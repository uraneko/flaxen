use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;

use crate::console::winsize::winsize;
use crate::layout::Layout;
use crate::render_pipeline;
use crate::space::{
    area_conflicts, between, border::Border, border_fit, padding::Padding, Area, Pos,
};
use crate::themes::Style;

use super::Property;
use super::{ComponentTreeError, SpaceError};
use super::{Term, Text};

/// Container objects are direct children of the Term object
/// and direct parents of the Text objects
#[derive(Debug, Default)]
pub struct Container {
    /// the layer of this container in terminal
    /// decide which container takes render priority in case of conflict
    /// think of it like css z-index
    pub layer: u8,
    /// unique id
    pub id: [u8; 2],
    /// children Text objects
    pub items: Vec<Text>,
    /// width
    pub w: u16,
    /// height
    pub h: u16,
    /// origin point x coordinate
    pub x0: u16,
    /// origin point y coordinate
    pub y0: u16,
    /// border value
    pub border: Border,
    /// padding value
    pub padding: Padding,
    // the following field has now become part of properties
    /// border style
    pub bstyle: String,
    pub layout: Layout,
    pub properties: HashMap<&'static str, Property>,
    pub attributes: HashSet<&'static str>,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Container {
    /// takes an id, origin coords, a width, a height, a border and padding
    /// returns a new Container
    pub fn new(
        id: [u8; 2],
        x0: u16,
        y0: u16,
        w: u16,
        h: u16,
        border: Border,
        padding: Padding,
    ) -> Container {
        Container {
            id,
            w,
            items: vec![],
            h,
            x0,
            layer: 0,
            layout: Layout::Canvas,
            y0,
            border,
            padding,
            bstyle: "".to_string(),
            properties: HashMap::new(),
            attributes: HashSet::new(),
        }
    }

    // TODO: bstyle, vstyle and layer should be properties

    // fn with_layer(id: [u8; 2], layer: u8) -> Self {
    //     Container {
    //         layer,
    //         id,
    //         w: 37,
    //         h: 5,
    //         x0: 5,
    //         y0: 2,
    //         ..Default::default()
    //     }
    // }

    /// changes the border style of this container
    pub fn bstyle(&mut self, style: &Style) {
        self.bstyle = style.style();
    }

    /// returns the id of the parent term of this container
    pub fn parent(&self) -> u8 {
        self.id[0]
    }

    // called on auto and base input/nonedit initializers
    /// checks for the validity of a text object's area before creating it
    pub(super) fn assign_valid_text_area(
        &self, // container
        text: &Text,
    ) -> Result<(), SpaceError> {
        let [x0, y0] = [text.x0, text.y0];
        let [w, h] = text.decorate();

        // check if new area is bigger than parent container area
        if self.w * self.h < w * h
            || x0 > self.w
            || y0 > self.h
            || w > self.w
            || h > self.h
            || x0 + w > self.w
            || y0 + h > self.h
        {
            // println!("0\r\n{x0} + {w} > {}\r\n{y0} + {h} > {}", self.w, self.h);
            return Err(SpaceError::AreaOutOfBounds);
        }

        let mut e = 0;

        self.items.iter().for_each(|t| {
            let [top, right, bottom, left] =
                area_conflicts(x0, y0, text.w, text.h, t.x0, t.y0, t.w, t.h);
            // conflict case
            if (left > 0 || right < 0) && (top > 0 || bottom < 0) {
                // TODO: actually handle overlay logic
                let e = self.shift_no_overlay(top, right, bottom, left);
                if e != 0 {}
            }
        });

        if e == 1 {
            // println!("1");
            return Err(SpaceError::AreaOutOfBounds);
        }

        Ok(())
    }

    // /// makes sure that text objects are properly positioned by moving them until they don't overlap when overlay is off
    // fn shift_text_area(&self, text: &mut Text) -> Result<(), SpaceError> {
    //     Ok(())
    // }
}
