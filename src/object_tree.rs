use crate::render_pipeline;
use crate::space::{area_conflicts, between, Border, Padding, SpaceAwareness};
use crate::termbuf::winsize;
use crate::themes::Style;

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ObjectTree {
    terms: Vec<Term>,
    active: u8,
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
            active: 0,
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

    pub fn make_active(&mut self, id: u8) -> Result<(), TreeErrors> {
        if self.has_term(id) {
            self.active = id;

            return Ok(());
        }

        Err(TreeErrors::BadID)
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

#[derive(Debug, Default)]
pub struct Term {
    pub overlay: bool,
    pub id: u8,
    pub cache: HashMap<String, Vec<Vec<Option<char>>>>,
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

impl Term {
    pub fn new(id: u8) -> Self {
        let ws = winsize::from_ioctl();

        Term {
            id,
            w: ws.cols(),
            h: ws.rows(),
            registry: HashSet::from(["Core"]),
            padding: Padding::None,
            border: Border::Uniform('*'),
            overlay: false,
            cache: HashMap::new(),
            cx: 0,
            cy: 0,
            containers: vec![],
            active: None,
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
                let [right, left, top, bottom] = area_conflicts(x0, y0, w, h, c.x0, c.y0, c.w, c.h);
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

    // /// makes sure that container objects are properly positioned by moving them until they don't overlap when overlay is off
    // pub fn shift_container_area(&self, text: &mut Text) -> Result<(), SpaceError> {
    //     Ok(())
    // }
}

impl Term {
    /// makes the text object with the given id the term's current active object
    /// calls sync_cursor
    pub fn make_active(&mut self, id: [u8; 3], writer: &mut StdoutLock) -> Result<(), TreeErrors> {
        if !self.has_input(&id) {
            return Err(TreeErrors::BadID);
        }
        self.active = Some(id);
        self.sync_cursor(writer);

        Ok(())
    }

    /// DEPRECATED
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

    /// syncs the position of the cursor in the term display to match the data in the backend
    pub fn sync_cursor(&mut self, writer: &mut StdoutLock) -> Result<(), TreeErrors> {
        let id = self.active.unwrap();

        let Ok([cx, cy]) = self.locate_text(&id) else {
            return Err(TreeErrors::BadID);
        };

        let [cx, cy] = match id[2] % 2 == 0 {
            true => {
                let t = self.input_ref(&id).unwrap();
                [t.ax0 + t.cx, t.ay0 + t.cy]
            }
            false => {
                let t = self.nonedit_ref(&id).unwrap();
                [t.ax0 + t.cx, t.ay0 + t.cy]
            }
        };

        self.cx = cx;
        self.cy = cy;

        let pos = format!("\x1b[{};{}f", self.cy, self.cx);
        _ = writer.write(pos.as_bytes());

        Ok(())
    }

    // returns immutable references to all text objects that have had interactions since the last event loop
    pub fn changed(&self) -> Vec<&Text> {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.change > 1))
            .flatten()
            .collect()
    }

    // returns mutable references to all text objects that have had interactions since the last event loop
    pub fn changed_mut(&mut self) -> Vec<&mut Text> {
        self.containers
            .iter_mut()
            .map(|c| c.items.iter_mut().filter(|t| t.change > 1))
            .flatten()
            .collect()
    }

    // resets all interactible objects' interactions value to 0
    // call this after every iteration of a program's event loop
    pub fn reset_changed(&mut self) {
        self.changed_mut().iter_mut().for_each(|t| {
            t.change = 0;
        });
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

    // calculates the absolute origin of a text object
    fn calc_text_abs_ori(
        &self,
        id: &[u8; 2],
        ori: &[u16; 2],
        ib: &Border,
        ip: &Padding,
    ) -> [u16; 2] {
        let [ix0, iy0] = ori;
        let Some(cont) = self.container_ref(&id) else {
            unreachable!("the container was already validated before getting here")
        };
        let [_, cpol, cpot, _, _, cpil, cpit, _] = render_pipeline::spread_padding(&cont.padding);
        let cb = if let Border::None = cont.border { 0 } else { 1 };

        let [_, ipol, ipot, _, _, ipil, ipit, _] = render_pipeline::spread_padding(&ip);
        let ib = if let Border::None = ib { 0 } else { 1 };

        [
            cpol + cb + cpil + cont.x0 + ipol + ib + ipil + ix0 + 1,
            cpot + cb + cpit + cont.y0 + ipot + ib + ipit + iy0,
        ]
    }

    pub fn input(
        &mut self,
        id: &[u8],
        name: &str,
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

        let [ax0, ay0] = self.calc_text_abs_ori(&[id[0], id[1]], &[x0, y0], &border, &padding);

        let mut cont = self.container_ref_mut(&[id[0], id[1]]).unwrap();

        let input = Text::new(
            [id[0], id[1], id[2]],
            name,
            x0,
            y0,
            ax0,
            ay0,
            w,
            h,
            &[],
            true,
            border,
            padding,
        );

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
        interactible: bool,
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

        let [ax0, ay0] = self.calc_text_abs_ori(&[id[0], id[1]], &[x0, y0], &border, &padding);

        let mut cont = self.container_ref_mut(&[id[0], id[1]]).unwrap();

        let nonedit = Text::new(
            [id[0], id[1], id[2]],
            "",
            x0,
            y0,
            ax0,
            ay0,
            w,
            h,
            value,
            interactible,
            border,
            padding,
        );

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

    /// returns a result of the active text object id
    /// or an error if it doesn't exist
    pub fn active(&self) -> Result<[u16; 2], TreeErrors> {
        // if self.active.is_none() {
        //     return Err(TreeErrors::BadID);
        // }

        let id = self.active.unwrap_or(return Err(TreeErrors::BadID));

        match id[2] % 2 == 0 {
            true => {
                let t = self.input_ref(&id).unwrap();
                Ok([t.ax0, t.ay0])
            }
            false => {
                let t = self.nonedit_ref(&id).unwrap();
                Ok([t.ax0, t.ay0])
            }
        }
    }

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

    /// returns whether the term has a container with the provided id
    pub fn has_container(&self, id: &[u8; 2]) -> bool {
        self.containers.iter().find(|c| c.id == *id).is_some()
    }

    /// returns whether any container in the term has an input with the provided id
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

    /// returns whether any container in the term has an noneditable with the provided id
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
        // NOTE: this method does not check the validity of the provided term id

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
        // NOTE: this method does not check the validity of the provided term and container ids
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
        // NOTE: this method does not check the validity of the provided term and container ids
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

#[derive(Debug, Default)]
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
    pub bstyle: String,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
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
            overlay: false,
            items: vec![],
            h,
            x0,
            layer: 0,
            registry: HashSet::new(),
            y0,
            border,
            padding,
            bstyle: "".to_string(),
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

    /// changes the border style of this container
    pub fn bstyle(&mut self, style: &Style) {
        self.bstyle = style.style();
    }

    /// returns the id of the parent term of this container
    pub fn parent(&self) -> u8 {
        self.id[0]
    }

    /// add new permit to the permit registry of this container
    pub fn permit<P>(&mut self) {
        self.registry.insert(type_name::<P>());
    }

    /// removes a permit from the registry of this container
    pub fn revoke<P>(&mut self) -> bool {
        self.registry.remove(type_name::<P>())
    }

    /// checks whether this container's permit registry has the provided permit
    pub fn has_permit<P>(&self) -> bool {
        self.registry.contains(type_name::<P>())
    }

    // called on auto and base input/nonedit initializers
    /// checks for the validity of a text object's area before creating it
    pub fn assign_valid_text_area(
        &self, // container
        text: &Text,
    ) -> Result<(), SpaceError> {
        let [x0, y0] = [text.x0, text.y0];
        let [w, h] = text.decorate();

        // check if new area is bigger than parent container area
        // FIXME: the first area check is wrong
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
                let [right, left, top, bottom] = area_conflicts(x0, y0, w, h, t.x0, t.y0, t.w, t.h);
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

    // /// makes sure that text objects are properly positioned by moving them until they don't overlap when overlay is off
    // fn shift_text_area(&self, text: &mut Text) -> Result<(), SpaceError> {
    //     Ok(())
    // }
}

#[derive(Debug, Default)]
pub struct Text {
    pub layer: u8,
    pub name: String,
    pub id: [u8; 3],
    pub temp: Vec<Option<char>>,
    pub value: Vec<Option<char>>,
    pub hicu: usize,
    pub w: u16,
    pub h: u16,
    pub cx: u16,
    pub cy: u16,
    pub x0: u16,
    pub y0: u16,
    pub ax0: u16,
    pub ay0: u16,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
    pub change: u8,
    pub bstyle: String,
    pub vstyle: String,
}

// NOTE: Inputs can only have pair IDs
// while NonEdits can only have odd IDs
impl Text {
    pub fn new(
        id: [u8; 3],
        name: &str,
        x0: u16,
        y0: u16,
        ax0: u16,
        ay0: u16,
        w: u16,
        h: u16,
        value: &[Option<char>],
        interactible: bool,
        border: Border,
        padding: Padding,
    ) -> Text {
        Text {
            id,
            name: name.to_string(),
            w,
            h,
            temp: vec![],
            hicu: 0,
            x0,
            y0,
            ax0,
            ay0,
            change: interactible.then_some(1).unwrap_or(0),
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
            registry: HashSet::new(),
            layer: 0,
            vstyle: "".to_string(),
            bstyle: "".to_string(),
        }
    }

    pub fn vstyle(&mut self, style: &Style) {
        self.vstyle = style.style();
    }

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

    /// returns the id of the parent container of this container
    pub fn parent(&self) -> [u8; 2] {
        [self.id[0], self.id[1]]
    }

    /// add new permit to the permit registry of this container
    pub fn permit<P>(&mut self) {
        self.registry.insert(type_name::<P>());
    }

    /// removes a permit from the registry of this container
    pub fn revoke<P>(&mut self) -> bool {
        self.registry.remove(type_name::<P>())
    }

    /// checks whether this container's permit registry has the provided permit
    pub fn has_permit<P>(&self) -> bool {
        self.registry.contains(type_name::<P>())
    }
}

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
