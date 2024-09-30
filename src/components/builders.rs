use crate::components::{Container, Term, Text};
use crate::layout::Layout;
use crate::space::{border::Border, padding::Padding, Area, Pos};

// pass the meta series to the component making methods in tree and term

#[derive(Debug, Clone)]
pub struct TermMeta {
    layout: Layout,
    area: Area,
    id: u8,
}

impl TermMeta {
    fn new() -> Self {
        Self {
            layout: Layout::Flex { direction: 'r' },
            area: Area::Zero,
            id: 0,
        }
    }

    fn id(mut self, id: u8) -> Self {
        self.id = id;
        self
    }

    fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    fn area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }

    fn clear(self) -> Self {
        Self::new()
    }

    pub(super) fn term(&mut self) -> Term {
        Term {
            id: {
                let id = self.id;
                self.bump_id();
                id
            },
            layout: self.layout.clone(),
            w: self.area.width().unwrap(),
            h: self.area.height().unwrap(),
            ..Term::default()
        }
    }

    fn bump_id(&mut self) {
        self.id += 1;
    }
}

#[derive(Debug, Clone)]
pub struct ContainerMeta {
    layer: u8,
    tid: u8,
    cid: u8,
    border: Border,
    padding: Padding,
    area: Area,
    layout: Layout,
    hpos: Pos,
    vpos: Pos,
}

impl ContainerMeta {
    fn new() -> Self {
        Self {
            layer: 0,
            tid: 0,
            cid: 0,
            padding: Padding::None,
            border: Border::None,
            area: Area::Fill,
            hpos: Pos::Center,
            vpos: Pos::Center,
            layout: Layout::Flex { direction: 'r' },
        }
    }

    fn overlay(mut self, overlay: bool) -> Self {
        self
    }

    fn layer(mut self, layer: u8) -> Self {
        self.layer = layer;
        self
    }

    fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    fn area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }

    fn hpos(mut self, hpos: Pos) -> Self {
        self.hpos = hpos;
        self
    }

    fn vpos(mut self, vpos: Pos) -> Self {
        self.vpos = vpos;
        self
    }

    fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    fn bump_tid(&mut self, id: u8) {
        self.tid = id;
    }

    fn bump_cid(&mut self, id: u8) {
        self.cid = id
    }

    fn id(&self) -> [u8; 2] {
        [self.tid, self.cid]
    }

    pub(super) fn container(&mut self) -> Container {
        Container {
            id: self.id(),
            layout: self.layout.clone(),
            w: self.area.width().unwrap(),
            h: self.area.height().unwrap(),
            ..Container::default()
        }
    }

    fn clear(self) -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct InputMeta {
    layer: u8,
    tid: u8,
    cid: u8,
    iid: u8,
    border: Border,
    padding: Padding,
    area: Area,
    hpos: Pos,
    vpos: Pos,
}

impl InputMeta {
    fn new() -> Self {
        Self {
            layer: 0,
            tid: 0,
            cid: 0,
            iid: 0,
            padding: Padding::None,
            border: Border::None,
            area: Area::Fill,
            hpos: Pos::Center,
            vpos: Pos::Center,
        }
    }

    fn layer(mut self, layer: u8) -> Self {
        self.layer = layer;
        self
    }

    fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    fn area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }

    fn hpos(mut self, hpos: Pos) -> Self {
        self.hpos = hpos;
        self
    }

    fn vpos(mut self, vpos: Pos) -> Self {
        self.vpos = vpos;
        self
    }

    fn clear(self) -> Self {
        Self::new()
    }

    pub fn input(&mut self) -> Text {
        Text {
            id: {
                let id = self.iid();
                self.bump_iid();
                id
            },
            w: self.area.width().unwrap_or(0),
            h: self.area.height().unwrap_or(0),
            ..Text::default()
        }
    }

    fn bump_tid(&mut self) {
        self.tid += 1;
    }

    fn bump_cid(&mut self) {
        self.cid += 1;
    }

    fn bump_iid(&mut self) {
        self.iid += 2;
    }

    pub(super) fn cid(&self) -> [u8; 2] {
        [self.tid, self.cid]
    }

    pub(super) fn iid(&self) -> [u8; 3] {
        [self.tid, self.cid, self.iid]
    }
}

#[derive(Debug, Clone)]
pub struct NonEditMeta {
    layer: u8,
    tid: u8,
    cid: u8,
    neid: u8,
    border: Border,
    padding: Padding,
    area: Area,
    hpos: Pos,
    vpos: Pos,
}

impl NonEditMeta {
    fn new() -> Self {
        Self {
            layer: 0,
            tid: 0,
            cid: 0,
            neid: 0,
            padding: Padding::None,
            border: Border::None,
            area: Area::Fill,
            hpos: Pos::Center,
            vpos: Pos::Center,
        }
    }

    fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    fn area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }

    fn hpos(mut self, hpos: Pos) -> Self {
        self.hpos = hpos;
        self
    }

    fn vpos(mut self, vpos: Pos) -> Self {
        self.vpos = vpos;
        self
    }

    fn clear(self) -> Self {
        Self::new()
    }

    pub fn nonedit(&mut self, value: Vec<Option<char>>) -> Text {
        Text {
            value,
            id: {
                let id = self.neid();
                self.bump_neid();
                id
            },
            w: self.area.width().unwrap_or(0),
            h: self.area.height().unwrap_or(0),
            ..Default::default()
        }
    }

    fn bump_tid(&mut self) {
        self.tid += 1;
    }

    fn bump_cid(&mut self) {
        self.cid += 1;
    }

    fn bump_neid(&mut self) {
        self.neid += 2;
    }

    pub(super) fn cid(&self) -> [u8; 2] {
        [self.tid, self.cid]
    }

    pub(super) fn neid(&self) -> [u8; 3] {
        [self.tid, self.cid, self.neid]
    }
}
