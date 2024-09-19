use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;

use crate::console::winsize::winsize;
use crate::render_pipeline;
use crate::space::{area_conflicts, between, border_fit, Border, Padding};
use crate::themes::Style;

use super::Property;
use super::{ComponentTreeError, SpaceError};
use super::{Container, Text};

/// Term object that is basically the overall wrapper around back end for the terminal display
#[derive(Debug, Default)]
pub struct Term {
    /// are containers in this Term allowed to overlap with each other
    pub overlay: bool,
    /// this Term's id
    pub id: u8,
    /// the width of the terminal window
    pub w: u16,
    /// the height of the terminal window
    pub h: u16,
    /// the current terminal cursor x coordinate
    pub cx: u16,
    /// the current terminal cursor y coordinate
    pub cy: u16,
    /// a vector of all the Containers inside this Term
    pub containers: Vec<Container>,
    // pub border: Border,
    // pub padding: Padding,
    /// the active Text object of this Term
    /// it is the Text object that the Term recognizes the user to be interacting with currently
    pub active: Option<[u8; 3]>,
    pub properties: HashMap<&'static str, Property>,
}

impl Term {
    /// returns a new term that holds the provided id
    ///
    /// # Examples
    /// ```
    /// let term = Term::new(0);
    /// ```
    ///
    /// # Errors
    ///
    /// the recommended way of creating a Term when a program uses more than 1 Term is to call the ComponentTree method term(id: u8)
    /// the term method would always validate the the new id before creating a term inside the tree
    /// if this function is called alongside tree's push_term() method then validating this term's
    /// id becomes the caller's job
    pub fn new(id: u8) -> Self {
        let ws = winsize::from_ioctl();

        Term {
            id,
            w: ws.cols(),
            h: ws.rows(),
            // padding: Padding::None,
            // border: Border::Uniform('*'),
            overlay: false,
            cx: 0,
            cy: 0,
            containers: vec![],
            active: None,
            properties: HashMap::new(),
        }
    }

    // since overlay is not implemented yet, this doesn't assign anything but just checks that the
    // area is valid
    // called on container auto and basic initializers
    pub(super) fn assign_valid_container_area(
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
                let [top, right, bottom, left] =
                    area_conflicts(x0, y0, cont.w, cont.h, c.x0, c.y0, c.w, c.h);
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
    /// places cursor in the new position by calling sync_cursor
    pub fn make_active(&mut self, id: &[u8; 3]) -> Result<(), ComponentTreeError> {
        let condition = match id[2] % 2 == 0 {
            true => self.has_input(&id),
            false => self.has_nonedit(&id) && self.nonedit_ref(&id).unwrap().change > 0,
        };

        if !condition {
            return Err(ComponentTreeError::BadID);
        }

        self.active = Some(*id);
        self.sync_cursor();

        Ok(())
    }

    /// syncs the position of the cursor in the term display to match the data in the backend
    pub fn sync_cursor(&mut self) -> Result<(), ComponentTreeError> {
        let id = self.active.unwrap();
        let text = if id[2] % 2 == 0 {
            self.input_ref(&id)
        } else {
            self.nonedit_ref(&id)
        }
        .unwrap();

        let [cx, cy] = [text.ax0 + text.cx, text.ay0 + text.cy];

        self.cx = cx;
        self.cy = cy;

        Ok(())
    }

    /// returns a result of the active text object absolute orign coords
    /// or an error if it doesn't exist
    pub fn active(&self) -> Result<[u16; 2], ComponentTreeError> {
        // if self.active.is_none() {
        //     return Err(ComponentTreeError::BadID);
        // }

        // BUG: same bug unwrap_or skips unwrap and automaticall does or in tests
        // let id = self.active.unwrap_or(return Err(ComponentTreeError::BadID));
        let id = match self.active {
            Some(id) => id,
            None => return Err(ComponentTreeError::BadID),
        };

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

    /// returns immutable references to all text objects that can be interacted with
    pub fn interactables(&self) -> Vec<&Text> {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.change > 0))
            .flatten()
            .collect()
    }

    /// returns an Optional of the next user interactable object
    /// the next interactable is either the next inside the current container
    /// or the first interactable inside the next container
    /// or the first container's first interactable if the current one is the last of all
    pub fn interactable_next(&self) -> Option<[u8; 3]> {
        if self.active.is_none() {
            return None;
        }

        let interactables = self.interactables();

        let pos = interactables
            .iter()
            .position(|t| t.id == self.active.unwrap());

        assert!(pos.is_some());

        if pos.unwrap() == interactables.len() - 1 {
            return Some(interactables[0].id);
        }

        let pos = pos.unwrap();

        Some(interactables[(pos + 1) as usize].id)
    }

    /// returns an Optional of the prev user interactable object
    /// the prev interactable is either the prev inside the current container
    /// or the last interactable inside the prev container
    /// or the last container's last interactable if the current one is the first of all
    pub fn interactable_prev(&self) -> Option<[u8; 3]> {
        if self.active.is_none() {
            return None;
        }

        let interactables = self.interactables();

        let pos = interactables
            .iter()
            .position(|t| t.id == self.active.unwrap());

        assert!(pos.is_some());

        if pos.unwrap() == 0 {
            return Some(interactables[interactables.len() - 1].id);
        }

        let pos = pos.unwrap();

        Some(interactables[(pos - 1) as usize].id)
    }

    /// returns immutable references to all text objects that have had interactions since the last event loop
    pub fn changed(&self) -> Vec<&Text> {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.change > 1))
            .flatten()
            .collect()
    }

    /// returns mutable references to all text objects that have had interactions since the last event loop
    pub fn changed_mut(&mut self) -> Vec<&mut Text> {
        self.containers
            .iter_mut()
            .map(|c| c.items.iter_mut().filter(|t| t.change > 1))
            .flatten()
            .collect()
    }

    /// resets all interactable objects' interactions value to 0
    /// call this after every iteration of a program's event loop
    // BUG: this break the active object rendering for some reason
    pub fn reset_changed(&mut self) {
        self.changed_mut().iter_mut().for_each(|t| {
            t.change = 1;
        });
    }
}

impl Term {
    /// adds a new Container object to this Term's containers
    ///
    /// # Examples
    /// ```
    /// let mut term = Term::new(0);
    /// let res = term.container(&[0, 0], 3, 3, 34, 18, Border::Uniform('+'), Padding::None);
    /// assert!(res.is_ok());
    /// ```
    /// # Errors
    /// returns an error if any of the following condition are met
    /// - the provided id is not of len == 2
    /// - the provided id is already taken by a container inside this term
    /// - x0 > Term width or y0 > Term height
    /// - w(idth) > Term width or h(eight) > Term height
    /// - this new container area infringes on a pre existing container's area in this Term and
    /// overlay is turned off for the Term
    pub fn container(
        &mut self,
        id: &[u8],
        x0: u16,
        y0: u16,
        w: u16,
        h: u16,
        border: Border,
        padding: Padding,
    ) -> Result<(), ComponentTreeError> {
        if id.len() > 2 || self.has_container(&[id[0], id[1]]) {
            eprintln!("bad id");
            return Err(ComponentTreeError::BadID);
        }

        if !border_fit(&border, &padding, w, h) {
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        let cont = Container::new([id[0], id[1]], x0, y0, w, h, border, padding);

        if self.assign_valid_container_area(&cont).is_err() {
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        self.containers.push(cont);

        Ok(())
    }

    /// pushes an existing Container to this Term's container vector
    ///
    /// # Examples
    ///
    /// ## Failure
    ///
    /// ```
    /// let mut term = Term::new(0);
    /// // wrong Term id '1' instead of '0'
    /// let cont = Container::new(&[1, 0], 3, 3, 34, 18, Border::Uniform('+'), Padding::None);
    /// let Err(res) = term.push_container(cont) else { unreachable!("you should have been an
    /// error") };
    /// assert_eq!(res.0.id, [0, 1]);
    /// ```
    ///
    /// ```
    /// let mut term = Term::new(0);
    /// // container starting x coordinate of '11111' > Term width
    /// let cont = Container::new(&[0, 0], 11111, 3, 34, 18, Border::Uniform('+'), Padding::None);
    /// let Err(res) = term.push_container(cont) else { unreachable!("you should have been an
    /// error") };
    /// assert_eq!(res.1, ComponentTreeError::BoundsNotRespected);
    /// ```
    ///
    /// ## Success
    ///
    /// ```
    /// let mut term = Term::new(0);
    /// let cont = Container::new(&[0, 0], 3, 3, 34, 18, Border::Uniform('+'), Padding::None);
    /// assert!(term.push_container(cont).is_ok());
    /// ```
    ///
    /// # Errors
    /// this method error conditions are the same as the container() method
    /// in case of an error, the Container that was passed as an argument is returned alongside the
    /// error value
    pub fn push_container(&mut self, c: Container) -> Result<(), (Container, ComponentTreeError)> {
        if self.has_container(&c.id) {
            return Err((c, ComponentTreeError::IDAlreadyExists));
        }

        // NOTE: assign_valid_thing_area series of functions need to be split to 2 fns
        // validate_thing_area and reassign_valid_thing_area
        // this fn's case only needs the validate_thing_area part

        if self.assign_valid_container_area(&c).is_err() {
            return Err((c, ComponentTreeError::BoundsNotRespected));
        }

        self.containers.push(c);

        Ok(())
    }

    /// takes only term id and automatically assigns an id for the container
    /// returns the full new container id
    // pub fn container_auto(
    //     &mut self,
    //     id: u8,
    //     x0: u16,
    //     y0: u16,
    //     w: u16,
    //     h: u16,
    // ) -> Result<[u8; 2], ComponentTreeError> {
    //     /// this should actually fail
    //     if !self.has_term(id) {
    //         return Err(ComponentTreeError::ParentNotFound);
    //     }
    //
    //     let id = [id, self.assign_container_id(id)];
    //
    //     let term = self.term_mut(id[0]).unwrap();
    //
    //     if term.assign_valid_container_area(x0, y0, w, h).is_err() {
    //         return Err(ComponentTreeError::BoundsNotRespected);
    //     }
    //
    //     term.containers.push(Container::new(id, x0, y0, w, h));
    //
    //     Ok(id)
    // }

    // calculates the absolute origin of a text object in terminal display coordinates
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

    /// pushes an existing input Text object to a child container of this Term
    pub fn push_input(&mut self, i: Text) -> Result<(), (Text, ComponentTreeError)> {
        if !self.has_container(&[i.id[0], i.id[1]]) || self.has_input(&i.id) || i.id[2] % 2 != 0 {
            return Err((i, ComponentTreeError::BadID));
        }

        self.container_mut(&[i.id[0], i.id[1]])
            .unwrap()
            .items
            .push(i);

        Ok(())
    }

    /// ...
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
    ) -> Result<(), ComponentTreeError> {
        if id.len() > 3
            || id[2] % 2 != 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_input(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id: {:?}", id);
            return Err(ComponentTreeError::BadID);
        }

        if !border_fit(&border, &padding, w, h) {
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        let [ax0, ay0] = self.calc_text_abs_ori(&[id[0], id[1]], &[x0, y0], &border, &padding);

        let mut cont = self.container_mut(&[id[0], id[1]]).unwrap();

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
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        cont.items.push(input);

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the input
    /// returns the full new input id
    /// DONT USE FOR NOW
    // pub fn input_auto(&mut self, id: &[u8]) -> Result<[u8; 3], ComponentTreeError> {
    //     if id.len() > 2 {
    //         eprintln!("use self.input(id) instead");
    //         return Err(ComponentTreeError::BadID);
    //     }
    //
    //     if !self.has_container(&[id[0], id[1]]) {
    //         eprintln!("bad id");
    //         return Err(ComponentTreeError::ParentNotFound);
    //     }
    //
    //     let id = [id[0], id[1], self.assign_input_id(id[0], id[1])];
    //
    //     self.container_mut(&[id[0], id[1]])
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
        interactable: bool,
        border: Border,
        padding: Padding,
    ) -> Result<(), ComponentTreeError> {
        if id.len() > 3
            || id[2] % 2 == 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_nonedit(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id");
            return Err(ComponentTreeError::BadID);
        }

        if !border_fit(&border, &padding, w, h) {
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        if value.len() as u16 > w * h {
            eprintln!(
                "value of len {} too long for bounds w * h {}",
                value.len(),
                w * h
            );
            return Err(ComponentTreeError::BadValue);
        }

        let [ax0, ay0] = self.calc_text_abs_ori(&[id[0], id[1]], &[x0, y0], &border, &padding);

        let mut cont = self.container_mut(&[id[0], id[1]]).unwrap();

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
            interactable,
            border,
            padding,
        );

        if cont.assign_valid_text_area(&nonedit).is_err() {
            return Err(ComponentTreeError::BoundsNotRespected);
        }

        cont.items.push(nonedit);

        Ok(())
    }

    /// pushes provided non editable Text object into a the Container with the given id if it
    /// exists and the Text object is valid, otherwise returns the error and Text object instance
    pub fn push_nonedit(&mut self, ne: Text) -> Result<(), (Text, ComponentTreeError)> {
        if !self.has_container(&[ne.id[0], ne.id[1]]) || self.has_input(&ne.id) || ne.id[2] % 2 == 0
        {
            return Err((ne, ComponentTreeError::BadID));
        }

        self.container_mut(&[ne.id[0], ne.id[1]])
            .unwrap()
            .items
            .push(ne);

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the nonedit
    /// returns the full new nonedit id
    // pub fn nonedit_auto(&mut self, id: &[u8]) -> Result<[u8; 3], ComponentTreeError> {
    //     if id.len() > 2 {
    //         eprintln!("use self.nonedit(id) instead");
    //         return Err(ComponentTreeError::BadID);
    //     }
    //
    //     if !self.has_container(&[id[0], id[1]]) {
    //         eprintln!("bad id");
    //         return Err(ComponentTreeError::ParentNotFound);
    //     }
    //
    //     let id = [id[0], id[1], self.assign_nonedit_id(id[0], id[1])];
    //
    //     self.container_mut(&[id[0], id[1]])
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
    pub fn container_mut(&mut self, id: &[u8; 2]) -> Option<&mut Container> {
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
    pub fn input_mut(&mut self, id: &[u8; 3]) -> Option<&mut Text> {
        let Some(cont) = self.container_mut(&[id[0], id[1]]) else {
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
    pub fn nonedit_mut(&mut self, id: &[u8; 3]) -> Option<&mut Text> {
        let Some(cont) = self.container_mut(&[id[0], id[1]]) else {
            return None;
        };

        cont.items
            .iter_mut()
            .find(|input| input.id[2] % 2 != 0 && input.id == *id)
    }

    /// returns the number of containers inside this term
    pub fn clen(&self) -> usize {
        self.containers.len()
    }

    /// return the sum of all the text objects inside this term
    pub fn tlen(&self) -> usize {
        self.containers.iter().map(|c| c.items.len()).sum::<usize>()
    }

    /// return the sum of all the input text objects inside this term
    pub fn ilen(&self) -> usize {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.id[2] % 2 == 0).count())
            .sum::<usize>()
    }

    /// return the sum of all the noneditable text objects inside this term
    pub fn nelen(&self) -> usize {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.id[2] % 2 != 0).count())
            .sum::<usize>()
    }

    /// return the sum of all the interactable text objects inside this term
    pub fn chlen(&self) -> usize {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.change != 0).count())
            .sum::<usize>()
    }

    /// return the sum of all the non-interactable text objects inside this term
    pub fn nclen(&self) -> usize {
        self.containers
            .iter()
            .map(|c| c.items.iter().filter(|t| t.change == 0).count())
            .sum::<usize>()
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

    fn assign_container_id(&self, term: u8) -> u8 {
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

    fn assign_input_id(&self, term: u8, cont: u8) -> u8 {
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

    fn assign_nonedit_id(&self, term: u8, cont: u8) -> u8 {
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
