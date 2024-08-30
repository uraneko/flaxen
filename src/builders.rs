use std::collections::{HashMap, HashSet};

use crate::object_tree::*;
use crate::space_awareness::{Border, Padding};
use crate::winsize;

#[derive(Debug, Default)]
pub struct ObjectBuilder {
    pub overlay: bool,
    pub id: [u8; 3],
    pub edit: bool,
    pub cache: HashMap<&'static str, Vec<u8>>,
    pub buf: Vec<char>,
    pub w: u16,
    pub h: u16,
    pub layer: u8,
    pub cx: u16,
    pub cy: u16,
    pub containers: Vec<Container>,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn term(&self) -> Term {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, None);

        Term {
            buf,
            w: ws.cols(),
            h: ws.rows(),
            registry: self
                .registry
                .iter()
                .map(|s| *s)
                .collect::<HashSet<&'static str>>(),
            padding: self.padding,
            border: self.border,
            id: self.id[0],
            overlay: self.overlay,
            containers: vec![],
            cx: 0,
            cy: 0,
            cache: Default::default(),
        }
    }
    pub fn container(&self) -> Container {
        Container {
            w: self.w,
            h: self.h,
            layer: self.layer,
            registry: self
                .registry
                .iter()
                .map(|s| *s)
                .collect::<HashSet<&'static str>>(),
            padding: self.padding,
            border: self.border,
            id: [self.id[0], self.id[1]],
            overlay: self.overlay,
            items: vec![],
            x0: 0,
            y0: 0,
        }
    }
    pub fn text(&self) -> Text {
        Text {
            edit: self.edit,
            w: self.w,
            h: self.h,
            layer: self.layer,
            registry: self
                .registry
                .iter()
                .map(|s| *s)
                .collect::<HashSet<&'static str>>(),
            padding: self.padding,
            border: self.border,
            id: [self.id[0], self.id[1], self.id[2]],
            cx: self.cx,
            cy: self.cy,
            value: vec![],
            x0: 0,
            y0: 0,
        }
    }
}
