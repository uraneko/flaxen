use crate::builders::ObjectBuilder;
use crate::render_pipeline;
use crate::space_awareness::{between, conflicts, Border, Padding, Point, SpaceAwareness};
use crate::termbuf::winsize;

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::marker::PhantomData;

// TODO: text position, vertical/horizontal center, start or end

#[derive(Debug)]
pub struct Zero;

#[derive(Debug)]
pub struct ObjectTree {
    terms: Vec<Term>,
}

#[derive(Debug)]
pub enum TreeErrors {
    BadID,
    BadValue,
    IDExists,
    ParentNotFound,
    BoundsNotRespected,
}

impl ObjectTree {
    pub fn new() -> Self {
        Self {
            terms: vec![Term::new(0)],
        }
    }

    pub fn term(&mut self, id: u8) -> Result<(), TreeErrors> {
        if self.has_term(id) {
            eprintln!("bad id");
            return Err(TreeErrors::IDExists);
        }
        self.terms.push(Term::new(id));

        Ok(())
    }

    /// takes no id and automatically assigns an id for the term
    /// returns the new term id
    pub fn term_auto(&mut self) -> u8 {
        let id = self.assign_term_id();

        self.terms.push(Term::new(id));

        id
    }

    /// returns an optional immutable reference of the term with the provided id if it exists
    pub fn term_ref(&self, id: u8) -> Option<&Term> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// returns an optional mutable reference of the term with the provided id if it exists
    pub fn term_ref_mut(&mut self, id: u8) -> Option<&mut Term> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    // methods of the has_object series do not check for duplicate ids
    // because those are already being screened by earlier id assignment methods
    // and there is no way in the api to bypass those checks and push an object to the tree
    // which means that duplicate ids can never happen
    pub fn has_term(&self, term: u8) -> bool {
        self.terms.iter().find(|t| t.id == term).is_some()
    }

    pub fn assign_term_id(&self) -> u8 {
        let mut id = 0;
        for term in &self.terms {
            if term.id == id {
                id += 1;
            } else {
                break;
            }
        }

        id
    }
}

enum SpaceError {
    E1,
}

#[derive(Debug)]
pub struct Term {
    pub overlay: bool,
    pub id: u8,
    pub cache: HashMap<&'static str, Vec<u8>>,
    pub buf: Vec<Option<char>>,
    pub w: u16,
    pub h: u16,
    pub cx: u16,
    pub cy: u16,
    pub containers: Vec<Container>,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
    pub active: Option<[u8; 3]>,
}

impl Default for Term {
    fn default() -> Term {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, None);

        Term {
            buf,
            w: ws.cols(),
            h: ws.rows(),
            registry: HashSet::from(["Core"]),
            active: None,
            padding: Padding::None,
            border: Border::None,
            overlay: false,
            id: 0,
            cache: HashMap::new(),
            cx: 0,
            cy: 0,
            containers: vec![],
        }
    }
}

impl Term {
    pub fn new(id: u8) -> Self {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, None);

        Term {
            id,
            buf,
            w: ws.cols(),
            h: ws.rows(),
            registry: HashSet::from(["Core"]),
            padding: Padding::None,
            border: Border::Uniform('*'),
            ..Default::default()
        }
    }

    pub fn from_builder(ob: &ObjectBuilder) -> Self {
        ob.term()
    }

    // prints the buffer, respecting width and height
    pub fn print_buf(&self) {
        for idxr in 0..self.buf.len() / self.w as usize {
            print!("\r\n");
            for idxc in 0..self.buf.len() / self.h as usize {
                print!("{:?}", self.buf[idxr]);
            }
        }
    }

    /// adds a Permit type to the term's registry
    pub fn permit<P>(&mut self) {
        self.registry.insert(type_name::<P>());
    }

    /// removes a Permit type to the term's registry
    pub fn revoke<P>(&mut self) -> bool {
        self.registry.remove(type_name::<P>())
    }

    /// checks if term has a Permit in its registry
    pub fn has_permit<P>(&self) -> bool {
        self.registry.contains(type_name::<P>())
    }

    // NOTE: for now gonna ignore overlay totally

    // called on container auto and basic initializers
    pub fn assign_valid_container_area(
        &self, // term
        cont: &Container,
        // layer: u8,
    ) -> Result<(), SpaceError> {
        let [x0, y0] = [cont.x0, cont.y0];
        let [w, h] = cont.decorate();

        // check if new area + padding + border is bigger than term area
        // FIX: the first area check is wrong
        // it should be:
        // if overlay in parent is on then current check
        // else parent area - all children area check against new container area
        if self.w * self.h < w * h
            || x0 > self.w
            || y0 > self.h
            || w > self.w
            || h > self.h
            || x0 + w > self.w
            || y0 + h > self.h
        {
            return Err(SpaceError::E1);
        }

        let mut e = 0;

        self.containers.iter().for_each(|c| {
            if e == 0 {
                let [right, left, top, bottom] = conflicts(x0, y0, w, h, c.x0, c.y0, c.w, c.h);
                // conflict case
                if (left > 0 || right < 0) && (top > 0 || bottom < 0) {
                    // TODO: actually handle overlay logic
                    e = 1;
                }
            }
        });

        if e == 1 {
            return Err(SpaceError::E1);
        }

        Ok(())
    }

    pub fn make_active(&mut self, id: [u8; 3], writer: &mut StdoutLock) -> Result<(), TreeErrors> {
        if !self.has_input(&id) {
            return Err(TreeErrors::BadID);
        }
        self.active = Some(id);
        self.sync_cursor(writer);

        Ok(())
    }

    // TODO: dont render everything at every iteration
    // only render all objects before loop
    // then rerender what changes inside an iteration

    pub fn locate_text(&self, id: &[u8; 3]) -> Result<[u16; 2], TreeErrors> {
        if let (Some(cont), Some(input)) =
            (self.container_ref(&[id[0], id[1]]), self.input_ref(&id))
        {
            let [_, cpol, cpot, _, _, cpil, cpit, _] =
                render_pipeline::spread_padding(&cont.padding);
            let cb = if let Border::None = cont.border { 0 } else { 1 };

            let [_, ipol, ipot, _, _, ipil, ipit, _] =
                render_pipeline::spread_padding(&input.padding);
            let ib = if let Border::None = input.border {
                0
            } else {
                1
            };
            Ok([
                cpol + cb + cpil + cont.x0 + ipol + ib + ipil + input.x0 + 1 + input.cx,
                cpot + cb + cpit + cont.y0 + ipot + ib + ipit + input.y0 + input.cy,
            ])
        } else {
            return Err(TreeErrors::BadID);
        }
    }

    pub fn sync_cursor(&mut self, writer: &mut StdoutLock) -> Result<(), TreeErrors> {
        let id = self.active.unwrap();

        let Ok([cx, cy]) = self.locate_text(&id) else {
            return Err(TreeErrors::BadID);
        };

        self.cx = cx;
        self.cy = cy;

        let pos = format!("\x1b[{};{}f", self.cy, self.cx);
        _ = writer.write(pos.as_bytes());

        Ok(())
    }
}

impl Term {
    pub fn container(
        &mut self,
        id: &[u8],
        x0: u16,
        y0: u16,
        w: u16,
        h: u16,
        border: Border,
        padding: Padding,
    ) -> Result<(), TreeErrors> {
        if id.len() > 2 || self.has_container(&[id[0], id[1]]) {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        let cont = Container::new([id[0], id[1]], x0, y0, w, h, border, padding);

        if self.assign_valid_container_area(&cont).is_err() {
            return Err(TreeErrors::BoundsNotRespected);
        }

        self.containers.push(cont);

        Ok(())
    }

    // need to have space values already by the time we reach object and object auto series methods
    // NOTE: should auto not take an object builder

    /// takes only term id and automatically assigns an id for the container
    /// returns the full new container id
    // pub fn container_auto(
    //     &mut self,
    //     id: u8,
    //     x0: u16,
    //     y0: u16,
    //     w: u16,
    //     h: u16,
    // ) -> Result<[u8; 2], TreeErrors> {
    //     /// this should actually fail
    //     if !self.has_term(id) {
    //         return Err(TreeErrors::ParentNotFound);
    //     }
    //
    //     let id = [id, self.assign_container_id(id)];
    //
    //     let term = self.term_ref_mut(id[0]).unwrap();
    //
    //     if term.assign_valid_container_area(x0, y0, w, h).is_err() {
    //         return Err(TreeErrors::BoundsNotRespected);
    //     }
    //
    //     term.containers.push(Container::new(id, x0, y0, w, h));
    //
    //     Ok(id)
    // }

    pub fn input(
        &mut self,
        id: &[u8],
        x0: u16,
        y0: u16,
        w: u16,
        h: u16,
        border: Border,
        padding: Padding,
    ) -> Result<(), TreeErrors> {
        if id.len() > 3
            || id[2] % 2 != 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_input(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        let mut cont = self.container_ref_mut(&[id[0], id[1]]).unwrap();

        let input = Text::new([id[0], id[1], id[2]], x0, y0, w, h, &[], border, padding);

        if cont.assign_valid_text_area(&input).is_err() {
            return Err(TreeErrors::BoundsNotRespected);
        }

        cont.items.push(input);

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the input
    /// returns the full new input id
    /// DONT USE FOR NOW
    // pub fn input_auto(&mut self, id: &[u8]) -> Result<[u8; 3], TreeErrors> {
    //     if id.len() > 2 {
    //         eprintln!("use self.input(id) instead");
    //         return Err(TreeErrors::BadID);
    //     }
    //
    //     if !self.has_container(&[id[0], id[1]]) {
    //         eprintln!("bad id");
    //         return Err(TreeErrors::ParentNotFound);
    //     }
    //
    //     let id = [id[0], id[1], self.assign_input_id(id[0], id[1])];
    //
    //     self.container_ref_mut(&[id[0], id[1]])
    //         .unwrap()
    //         .items
    //         .push(Text::new(id));
    //
    //     Ok(id)
    // }

    pub fn nonedit(
        &mut self,
        id: &[u8],
        x0: u16,
        y0: u16,
        w: u16,
        h: u16,
        value: &[Option<char>],
        border: Border,
        padding: Padding,
    ) -> Result<(), TreeErrors> {
        if id.len() > 3
            || id[2] % 2 == 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_nonedit(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        if value.len() as u16 > w * h {
            eprintln!(
                "value of len {} too long for bounds w * h {}",
                value.len(),
                w * h
            );
            return Err(TreeErrors::BadValue);
        }

        let mut cont = self.container_ref_mut(&[id[0], id[1]]).unwrap();

        let nonedit = Text::new([id[0], id[1], id[2]], x0, y0, w, h, value, border, padding);

        if cont.assign_valid_text_area(&nonedit).is_err() {
            return Err(TreeErrors::BoundsNotRespected);
        }

        cont.items.push(nonedit);

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the nonedit
    /// returns the full new nonedit id
    // pub fn nonedit_auto(&mut self, id: &[u8]) -> Result<[u8; 3], TreeErrors> {
    //     if id.len() > 2 {
    //         eprintln!("use self.nonedit(id) instead");
    //         return Err(TreeErrors::BadID);
    //     }
    //
    //     if !self.has_container(&[id[0], id[1]]) {
    //         eprintln!("bad id");
    //         return Err(TreeErrors::ParentNotFound);
    //     }
    //
    //     let id = [id[0], id[1], self.assign_nonedit_id(id[0], id[1])];
    //
    //     self.container_ref_mut(&[id[0], id[1]])
    //         .unwrap()
    //         .items
    //         .push(Text::new(id));
    //
    //     Ok(id)
    // }

    /// returns an optional immutable reference of the container with the provided id if it exists
    pub fn container_ref(&self, id: &[u8; 2]) -> Option<&Container> {
        self.containers.iter().find(|c| &c.id == id)
    }

    /// returns an optional mutable reference of the container with the provided id if it exists
    pub fn container_ref_mut(&mut self, id: &[u8; 2]) -> Option<&mut Container> {
        self.containers.iter_mut().find(|c| &c.id == id)
    }

    /// returns an optional immutable reference of the input with the provided id if it exists
    pub fn input_ref(&self, id: &[u8; 3]) -> Option<&Text> {
        let Some(cont) = self.container_ref(&[id[0], id[1]]) else {
            return None;
        };

        cont.items
            .iter()
            .find(|input| input.id[2] % 2 == 0 && input.id == *id)
    }

    /// returns an optional mutable reference of the input with the provided id if it exists
    pub fn input_ref_mut(&mut self, id: &[u8; 3]) -> Option<&mut Text> {
        let Some(cont) = self.container_ref_mut(&[id[0], id[1]]) else {
            return None;
        };

        cont.items
            .iter_mut()
            .find(|input| input.id[2] % 2 == 0 && input.id == *id)
    }

    /// returns an optional immutable reference of the noneditable with the provided id if it exists
    pub fn nonedit_ref(&self, id: &[u8; 3]) -> Option<&Text> {
        let Some(cont) = self.container_ref(&[id[0], id[1]]) else {
            return None;
        };

        cont.items
            .iter()
            .find(|input| input.id[2] % 2 != 0 && input.id == *id)
    }

    /// returns an optional mutable reference of the noneditable with the provided id if it exists
    pub fn nonedit_ref_mut(&mut self, id: &[u8; 3]) -> Option<&mut Text> {
        let Some(cont) = self.container_ref_mut(&[id[0], id[1]]) else {
            return None;
        };

        cont.items
            .iter_mut()
            .find(|input| input.id[2] % 2 != 0 && input.id == *id)
    }

    pub fn has_container(&self, id: &[u8; 2]) -> bool {
        self.containers.iter().find(|c| c.id == *id).is_some()
    }

    pub fn has_input(&self, id: &[u8; 3]) -> bool {
        match self.container_ref(&[id[0], id[1]]) {
            Some(cont) => cont
                .items
                .iter()
                .find(|input| input.id[2] % 2 == 0 && input.id == *id)
                .is_some(),
            None => {
                eprintln!("no container with such id was found {:?}", &id[..2]);
                false
            }
        }
    }

    pub fn has_nonedit(&self, id: &[u8; 3]) -> bool {
        match self.container_ref(&[id[0], id[1]]) {
            Some(cont) => cont
                .items
                .iter()
                .find(|input| input.id[2] % 2 != 0 && input.id == *id)
                .is_some(),
            None => {
                eprintln!("no container with such id was found {:?}", &id[..2]);
                false
            }
        }
    }

    pub fn assign_container_id(&self, term: u8) -> u8 {
        // NOTE: this method should always be called inside another method/fn
        // that checks before calling this method that
        // the parent term with the given id exists

        let mut id = 0;
        for cont in &self.containers {
            if cont.id[1] == id {
                id += 1;
            } else {
                break;
            }
        }

        id
    }

    pub fn assign_input_id(&self, term: u8, cont: u8) -> u8 {
        // NOTE: this method should always be called inside another method/fn
        // that checks before calling this method that
        // the parent term and container with the given ids exist
        let cont = self.container_ref(&[term, cont]).unwrap();

        let mut id = 0;
        let mut iter = cont.items.iter().filter(|i| i.id[2] % 2 == 0);
        while let Some(item) = iter.next() {
            if item.id[2] == id {
                id += 2;
            } else {
                break;
            }
        }

        id
    }

    pub fn assign_nonedit_id(&self, term: u8, cont: u8) -> u8 {
        // NOTE: this method should always be called inside another method/fn
        // that checks before calling this method that
        // the parent term and container with the given ids exist
        let cont = self.container_ref(&[term, cont]).unwrap();

        let mut id = 0;
        let mut iter = cont.items.iter().filter(|i| i.id[2] % 2 != 0);
        while let Some(item) = iter.next() {
            if item.id[2] == id {
                id += 2;
            } else {
                break;
            }
        }

        id
    }
}

// TODO: need a way

#[derive(Debug)]
pub struct Container {
    pub overlay: bool,
    pub layer: u8,
    pub id: [u8; 2],
    pub items: Vec<Text>,
    pub w: u16,
    pub h: u16,
    pub x0: u16,
    pub y0: u16,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

// [IMPORTANT]NOTE:
// the Commissioner handles all ID matters
// he also handles all Events matters
// and all Space allocation matters
// he is the only one with access to the Term father
// TODO: commissioner needs to handle the space, id requests allocations

impl Default for Container {
    fn default() -> Self {
        Self {
            id: [0, 0],
            layer: 0,
            w: 37,
            h: 5,
            overlay: false,
            items: vec![],
            x0: 5,
            y0: 2,
            padding: Padding::None,
            border: Border::None,
            registry: HashSet::new(),
        }
    }
}

impl Container {
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
            h,
            x0,
            y0,
            border,
            padding,
            ..Default::default()
        }
    }

    pub fn with_layer(id: [u8; 2], layer: u8) -> Self {
        Container {
            layer,
            id,
            w: 37,
            h: 5,
            x0: 5,
            y0: 2,
            ..Default::default()
        }
    }

    pub fn from_builder(ob: &ObjectBuilder) -> Self {
        ob.container()
    }

    pub fn permit<P>(&mut self) {
        self.registry.insert(type_name::<P>());
    }

    pub fn revoke<P>(&mut self) -> bool {
        self.registry.remove(type_name::<P>())
    }

    pub fn has_permit<P>(&self) -> bool {
        self.registry.contains(type_name::<P>())
    }

    /// takes all items in container and makes a buffer of char values that correspond to how that
    /// container should look like in term display
    pub fn buffer(&self) -> Vec<char> {
        vec![]
    }

    // called on auto and base input/nonedit initializers
    pub fn assign_valid_text_area(
        &self, // container
        text: &Text,
    ) -> Result<(), SpaceError> {
        let [x0, y0] = [text.x0, text.y0];
        let [w, h] = text.decorate();

        // check if new area is bigger than parent container area
        // FIX: the first area check is wrong
        // it should be:
        // if overlay in parent is on then current check
        // else parent area - all children area check against new container area
        if self.w * self.h < w * h
            || x0 > self.w
            || y0 > self.h
            || w > self.w
            || h > self.h
            || x0 + w > self.w
            || y0 + h > self.h
        {
            return Err(SpaceError::E1);
        }

        let mut e = 0;

        self.items.iter().for_each(|t| {
            if e == 0 {
                let [right, left, top, bottom] = conflicts(x0, y0, w, h, t.x0, t.y0, t.w, t.h);
                // conflict case
                if (left > 0 || right < 0) && (top > 0 || bottom < 0) {
                    // TODO: actually handle overlay logic
                    e = 1;
                }
            }
        });

        if e == 1 {
            return Err(SpaceError::E1);
        }

        Ok(())
    }
}

// TODO: make the current Objects new implementations into Default impls
// then impl new with arguments

#[derive(Debug)]
pub struct Text {
    pub layer: u8,
    // editable content or not
    pub edit: bool,
    pub id: [u8; 3],
    pub value: Vec<Option<char>>,
    pub w: u16,
    pub h: u16,
    pub cx: u16,
    pub cy: u16,
    pub x0: u16,
    pub y0: u16,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
}

impl Default for Text {
    fn default() -> Self {
        Text {
            id: [0, 0, 0],
            w: 20,
            h: 3,
            x0: 5,
            y0: 3,
            value: vec![
                Some('h'),
                Some('e'),
                Some('l'),
                Some('l'),
                Some('o'),
                Some(' '),
                Some('t'),
                Some('e'),
                Some('r'),
                Some('m'),
            ],
            cx: 0,
            cy: 0,
            registry: HashSet::new(),
            border: Border::None,
            padding: Padding::None,
            edit: false,
            layer: 0,
        }
    }
}

// Inputs can only have pair IDs
// while NonEdits can only have odd IDs
impl Text {
    pub fn new(
        id: [u8; 3],
        x0: u16,
        y0: u16,
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
            x0,
            y0,
            border,
            padding,
            value: {
                let mut v = Vec::with_capacity((w * h) as usize);
                v.resize((w * h) as usize, None);
                v.extend_from_slice(value);

                v
            },
            ..Default::default()
        }
    }

    pub fn with_layer(id: [u8; 3], layer: u8) -> Self {
        Text {
            layer,
            id,
            w: 37,
            h: 5,
            x0: 5,
            y0: 2,
            ..Default::default()
        }
    }

    pub fn from_builder(ob: &ObjectBuilder) -> Self {
        ob.text()
    }

    pub fn permit<P>(&mut self) {
        self.registry.insert(type_name::<P>());
    }

    pub fn revoke<P>(&mut self) -> bool {
        self.registry.remove(type_name::<P>())
    }

    pub fn has_permit<P>(&self) -> bool {
        self.registry.contains(type_name::<P>())
    }
}

#[derive(Debug)]
pub enum IDError {
    ProgramIsUnique,
}

use std::io::StdoutLock;

struct InnerLogic;

#[cfg(test)]
mod tests0 {}

#[cfg(test)]
mod tests1 {
    use super::Term;

    #[test]
    fn test_raw_mode() {
        let _ = Term::new(0);

        println!("we are now inside raw mode");
        println!("we are now inside raw mode");
    }
}
