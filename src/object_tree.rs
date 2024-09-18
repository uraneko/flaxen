use crate::console::winsize::winsize;
use crate::render_pipeline;
use crate::space::{area_conflicts, between, border_fit, Border, Padding};
use crate::themes::Style;

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;
use std::marker::PhantomData;

struct Permit {
    i: &'static str,
    o: &'static str,
    s: &'static str,
}

impl Permit {
    fn from_impl<S, I, O>() -> Self {
        Self {
            s: type_name::<S>(),
            i: type_name::<I>(),
            o: type_name::<O>(),
        }
    }

    fn new() -> Self {
        Self {
            s: "",
            i: "",
            o: "",
        }
    }
}

type Styles = Vec<Style>;
type Registries = Vec<Permit>;
// a branch is a number of ids grouped together
// idx0 is the term id, [idx1, idx2] are the container ids and the remaining items are all text ids

type TIDs = Vec<u8>;
type CIDs = Vec<[u8; 2]>;
type TxIDs = Vec<[u8; 3]>;

struct Branch {
    value: &'static [u8],
    next: &'static Branch,
    prev: &'static Branch,
}

/// used when creating a new object
/// bestows all the passed authorities to the new object (Term/Container/Text) instance
/// this can only be called from inside the ComponentTree's term() or reinit() methods and Term's container(),
/// text() and reinit() methods
fn bestow_authorities(sio: &[Permit]) {}
/// used when reinitializing an object
/// revokes all the authorities of the given object (Term/Container/Text) instance
fn revoke_authorities() {}

#[derive(Debug)]
enum Property {
    String(String),
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    Vec(Vec<Property>),
    Term(Term),
    Container(Container),
    Text(Text),
    Map(HashMap<&'static str, Property>),
}

/// the wrpper struct holding all the program term objects
#[derive(Debug)]
pub struct ComponentTree {
    terms: Vec<Term>,
    active: u8,
    // TODO: active should become a property
    properties: HashMap<&'static str, Property>,
}

/// errors for ComponentTree operations
#[derive(Debug)]
pub enum ComponentTreeError {
    /// Obscure error; something about some id somewhere went wrong
    BadID,
    ///
    BadValue,
    /// when trying to assugn an ID that has already been assigned prior to this
    IDAlreadyExists,
    /// the parent object of some object that is being operated on was not found in this tree
    ParentNotFound,
    /// the space bounds rules were broken by some object trying to join this tree
    BoundsNotRespected,
}

impl ComponentTree {
    /// creates a new ComponentTree instance
    /// normally, this should only be used once in a crate
    /// # Examples
    /// ```
    /// let tree = ComponentTree::new();
    /// ```
    /// this automatically creates a new Term with the id value of 0 inside this new Tree
    pub fn new() -> Self {
        Self {
            terms: vec![Term::new(0)],
            active: 0,
            properties: HashMap::new(),
        }
    }

    /// pushes an existing Term to this tree's terms vector
    ///
    /// # Examples
    ///
    /// ## Failure
    ///
    /// ```
    /// let mut tree = ComponentTree::new();
    /// let term = Term::new(0);
    /// assert!(tree.push_term(term).is_err());
    /// ```
    ///
    /// ## Success
    /// ```
    /// let mut tree = ComponentTree::new();
    /// let term = Term::new(1);
    /// assert!(tree.push_term(term).is_ok());
    /// ```
    ///
    /// # Errors
    /// returns an error only if the new Term's id is already taken by another Term in this Tree
    ///
    pub fn push_term(&mut self, term: Term) -> Result<(), (Term, ComponentTreeError)> {
        if self.has_term(term.id) {
            return Err((term, ComponentTreeError::IDAlreadyExists));
        }
        self.terms.push(term);

        Ok(())
    }

    /// adds a new Term object to this tree
    /// takes an id value for the new Term
    /// # Errors
    ///
    /// this method returns an ComponentTreeError if the id provided is already being used by another
    /// Term in this tree
    pub fn term(&mut self, id: u8) -> Result<(), ComponentTreeError> {
        if self.has_term(id) {
            eprintln!("bad id");
            return Err(ComponentTreeError::IDAlreadyExists);
        }
        self.terms.push(Term::new(id));

        Ok(())
    }

    /// returns the active Term object id
    pub fn active(&self) -> u8 {
        self.active
    }

    /// changes the active Term of this tree
    /// the active term is the term that gets rendered
    ///
    /// # Errors
    ///
    /// returns an error if a Term with the provided id does not exist in this tree
    pub fn make_active(&mut self, id: u8) -> Result<(), ComponentTreeError> {
        if self.has_term(id) {
            self.active = id;

            return Ok(());
        }

        Err(ComponentTreeError::BadID)
    }

    /// takes no id and automatically assigns an id while adding a new Term
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
    pub fn term_mut(&mut self, id: u8) -> Option<&mut Term> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    // methods of the has_object series do not check for duplicate ids
    // because those are already being screened by earlier id assignment methods
    // and there is no way in the api to bypass those checks and push an object to the tree
    // which means that duplicate ids can never happen
    /// checks for the existence of a Term with the provided id inside this tree
    pub fn has_term(&self, term: u8) -> bool {
        self.terms.iter().find(|t| t.id == term).is_some()
    }

    fn assign_term_id(&self) -> u8 {
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

/// Term object that is basically the overall wrapper around back end for the terminal display
#[derive(Debug, Default)]
pub struct Term {
    /// are containers in this Term allowed to overlap with each other
    pub overlay: bool,
    /// this Term's id
    pub id: u8,
    /// a simple cache for all input Text objects inside this Term
    pub cache: HashMap<String, Vec<Vec<Option<char>>>>,
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
    /// a collection of the permits of this Term,
    /// the permits held by the instance should dictate which Events trait implementations it is allowed to call, but for now this does not really interfere with the rest of the crate
    pub scopes: HashSet<&'static str>,
    // pub border: Border,
    // pub padding: Padding,
    /// the active Text object of this Term
    /// it is the Text object that the Term recognizes the user to be interacting with currently
    pub active: Option<[u8; 3]>,
}

fn parse_permit(p: &'static str) -> &'static str {
    if p.contains(':') {
        return p.split("::").last().unwrap();
    }

    p
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
            scopes: HashSet::from(["Core"]),
            // padding: Padding::None,
            // border: Border::Uniform('*'),
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
        self.scopes.insert(parse_permit(type_name::<P>()));
    }

    /// removes a Permit type to the term's.scopes
    pub fn revoke<P>(&mut self) -> bool {
        self.scopes.remove(parse_permit(type_name::<P>()))
    }

    /// checks if term has a Permit in its.scopes
    pub fn has_permit<P>(&self) -> bool {
        self.scopes.contains(parse_permit(type_name::<P>()))
    }

    // since overlay is not implemented yet, this doesn't assign anything but just checks that the
    // area is valid
    // called on container auto and basic initializers
    fn assign_valid_container_area(
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

/// Container objects are direct children of the Term object
/// and direct parents of the Text objects
#[derive(Debug, Default)]
pub struct Container {
    /// allows overlaying inside this container
    pub overlay: bool,
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
    /// what implementation scopes this instance of Container is allowed to call
    pub scopes: HashSet<&'static str>,
    /// border value
    pub border: Border,
    /// padding value
    pub padding: Padding,
    /// border style
    pub bstyle: String,
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
            overlay: false,
            items: vec![],
            h,
            x0,
            layer: 0,
            scopes: HashSet::new(),
            y0,
            border,
            padding,
            bstyle: "".to_string(),
        }
    }

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

    /// add new permit to the permit registry of this container
    pub fn permit<P>(&mut self) {
        self.scopes.insert(parse_permit(type_name::<P>()));
    }

    /// removes a permit from the registry of this container
    pub fn revoke<P>(&mut self) -> bool {
        self.scopes.remove(parse_permit(type_name::<P>()))
    }

    /// checks whether this container's permit registry has the provided permit
    pub fn has_permit<P>(&self) -> bool {
        self.scopes.contains(parse_permit(type_name::<P>()))
    }

    // called on auto and base input/nonedit initializers
    /// checks for the validity of a text object's area before creating it
    fn assign_valid_text_area(
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
            // println!("0\r\n{x0} + {w} > {}\r\n{y0} + {h} > {}", self.w, self.h);
            return Err(SpaceError::E1);
        }

        let mut e = 0;

        self.items.iter().for_each(|t| {
            if e == 0 {
                let [top, right, bottom, left] =
                    area_conflicts(x0, y0, text.w, text.h, t.x0, t.y0, t.w, t.h);
                // conflict case
                if (left > 0 || right < 0) && (top > 0 || bottom < 0) {
                    // TODO: actually handle overlay logic
                    e = 1;
                }
            }
        });

        if e == 1 {
            // println!("1");
            return Err(SpaceError::E1);
        }

        Ok(())
    }

    // /// makes sure that text objects are properly positioned by moving them until they don't overlap when overlay is off
    // fn shift_text_area(&self, text: &mut Text) -> Result<(), SpaceError> {
    //     Ok(())
    // }
}

/// Text objects are direct children of the Container objects
/// and indirect children of the Term grand parent
#[derive(Debug, Default)]
pub struct Text {
    /// the layer of this Text inside its parent Container
    /// decide which Text takes render priority in case of conflict
    /// think of it like css z-index
    pub layer: u8,
    /// Text object name to be used for caching
    pub name: String,
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
    /// is this Text pbject changeable and if so did it have any changes to at least value or border
    pub change: u8,
    /// border style
    pub bstyle: String,
    /// value style
    pub vstyle: String,
}

// NOTE: Inputs can only have pair IDs
// while NonEdits can only have odd IDs
impl Text {
    /// creates a new Text objects
    /// takes most of Text's field values as arguments and returns a Text instance
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
        interactable: bool,
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
            change: interactable.then_some(1).unwrap_or(0),
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

    /// add new permit to the permit registry of this text
    pub fn permit<P>(&mut self) {
        self.scopes.insert(parse_permit(type_name::<P>()));
    }

    /// removes a permit from the registry of this text
    pub fn revoke<P>(&mut self) -> bool {
        self.scopes.remove(parse_permit(type_name::<P>()))
    }

    /// checks whether this text's permit.scopes has the provided permit
    pub fn has_permit<P>(&self) -> bool {
        self.scopes.contains(parse_permit(type_name::<P>()))
    }
}

#[cfg(test)]
mod object_tree {
    use super::{ComponentTree, ComponentTreeError, Term};

    #[test]
    fn active() {
        let mut tree = ComponentTree::new();
        assert!(tree.term(0).is_err());
        assert_eq!(tree.terms.len(), 1);

        tree.term(7);

        assert_eq!(tree.terms.len(), 2);

        assert_eq!(tree.active(), 0);
        tree.make_active(3);
        assert_eq!(tree.active(), 0);
        tree.make_active(7);
        assert_eq!(tree.active(), 7);
    }

    #[test]
    fn assign() {
        let mut tree = ComponentTree::new();
        tree.term_auto();
        assert!(tree.has_term(0));
        let term: &Term = tree.term_ref(0).unwrap();
        let term: &mut Term = tree.term_mut(0).unwrap();
        assert!(tree.term_ref(78).is_none());
        tree.term(1);
        tree.term(2);
        tree.term(4);
        assert_eq!(tree.assign_term_id(), 3);
    }
}

#[cfg(test)]
mod term {
    use super::{Container, Term, Text};

    #[test]
    fn registry() {
        let mut term = Term::new(7);

        struct A1;
        struct Core;
        assert_eq!(term.scopes.len(), 1);

        term.permit::<A1>();
        assert!(term.has_permit::<A1>());

        term.revoke::<A1>();
        assert!(!term.has_permit::<A1>());
        assert!(term.has_permit::<Core>());
    }

    #[test]
    fn area() {
        let mut term = Term::new(5);
        let mut c1 = Container::default();
        c1.w = 24;
        c1.h = 32;
        c1.x0 = 2;
        c1.y0 = 5;
        assert!(term.assign_valid_container_area(&c1).is_ok());
        c1.w = 8354;
        c1.h = 3;
        c1.x0 = 2;
        c1.y0 = 5;
        assert!(term.assign_valid_container_area(&c1).is_err());
        c1.w = 4;
        c1.h = 8324;
        c1.x0 = 2;
        c1.y0 = 5;
        assert!(term.assign_valid_container_area(&c1).is_err());
        c1.w = 4;
        c1.h = 3;
        c1.x0 = 8355;
        c1.y0 = 5;
        assert!(term.assign_valid_container_area(&c1).is_err());
        c1.w = 4;
        c1.h = 3;
        c1.x0 = 2;
        c1.y0 = 8653;
        assert!(term.assign_valid_container_area(&c1).is_err());
    }

    #[test]
    fn active() {
        let mut term = Term::new(5);
        let mut c = Container::default();
        c.x0 = 4;
        c.y0 = 5;
        c.id = [5, 1];
        let mut ne = Text::default();
        ne.x0 = 7;
        ne.y0 = 4;
        ne.id = [5, 1, 3];

        // when more area manipulation methods are written
        // return to this and make it a proper check of those methods
        [ne.ax0, ne.ay0] = [ne.x0 + c.x0, ne.y0 + c.y0];

        ne.change = 1;

        let id = ne.id;

        term.push_container(c);
        term.push_nonedit(ne);

        let res = term.make_active(&[5, 1, 8]);
        assert!(res.is_err());

        let res = term.make_active(&id);
        assert!(res.is_ok());
        assert_eq!(term.active.unwrap(), id);
        assert_eq!(term.active().unwrap(), [11, 9]);
    }

    use crate::space::{Border, Padding};

    #[test]
    fn cursor() {
        let mut term = Term::new(5);
        _ = term.container(&[0, 0], 56, 15, 35, 8, Border::None, Padding::None);
        _ = term.input(&[0, 0, 0], "", 1, 1, 23, 2, Border::None, Padding::None);

        let res = term.make_active(&[0, 0, 0]);
        assert_eq!([term.cx, term.cy], [56 + 1 + 1, 15 + 1]);
    }

    #[test]
    fn interactables() {
        let mut term = Term::new(0);
        _ = term.container(&[0, 0], 56, 15, 35, 18, Border::None, Padding::None);
        _ = term.input(&[0, 0, 0], "", 1, 1, 2, 2, Border::None, Padding::None);
        let res = term.input(&[0, 0, 2], "", 5, 5, 2, 2, Border::None, Padding::None);
        println!("{:?}", res);
        let res = term.nonedit(
            &[0, 0, 1],
            12,
            12,
            2,
            2,
            &[],
            true,
            Border::None,
            Padding::None,
        );
        println!("{:?}", res);

        let inters = term.interactables();
        assert_eq!(inters.len(), 3);

        term.make_active(&[0, 0, 1]);
        assert_eq!(term.interactable_next(), Some([0, 0, 0]));
        assert_eq!(term.interactable_prev(), Some([0, 0, 2]));

        assert_eq!(term.changed().len(), 0);
        // simulate value change
        term.input_mut(&[0, 0, 2]).unwrap().change = 2;
        assert_eq!(term.changed().len(), 1);
    }

    #[test]
    fn objects() {
        let mut term = Term::new(0);
        term.push_container(Container::default());
        term.container(&[0, 1], 56, 15, 35, 18, Border::None, Padding::None);
        assert_eq!(term.containers.len(), 2);
        term.push_input({
            let mut i = Text::default();
            i.change = 1;
            i
        });
        term.nonedit(
            &[0, 1, 1],
            12,
            12,
            2,
            2,
            &[],
            true,
            Border::None,
            Padding::None,
        );

        assert_eq!(term.tlen(), 2)
    }

    // test calc_text_abs_ori

    #[test]
    fn objects1() {
        let mut term = Term::new(0);
        term.push_container(Container::default());
        term.container(&[0, 1], 56, 15, 35, 18, Border::None, Padding::None);
        assert_eq!(term.containers.len(), 2);
        term.push_input({
            let mut i = Text::default();
            i.change = 1;
            i
        });
        term.nonedit(
            &[0, 1, 1],
            12,
            12,
            2,
            2,
            &[],
            true,
            Border::None,
            Padding::None,
        );

        assert_eq!(term.tlen(), 2);
    }

    #[test]
    fn objects_count() {
        let mut term = Term::new(0);

        term.container(&[0, 0], 5, 5, 10, 10, Border::None, Padding::None);
        term.container(&[0, 1], 15, 15, 10, 10, Border::None, Padding::None);
        term.container(&[0, 2], 25, 25, 10, 10, Border::None, Padding::None);

        term.input(&[0, 2, 0], "", 1, 2, 2, 2, Border::None, Padding::None);
        term.nonedit(
            &[0, 1, 1],
            2,
            2,
            2,
            2,
            &[],
            true,
            Border::None,
            Padding::None,
        );

        term.nonedit(
            &[0, 0, 1],
            1,
            1,
            2,
            2,
            &[],
            false,
            Border::None,
            Padding::None,
        );

        assert_eq!(term.tlen(), 3);
        assert_eq!(term.ilen(), 1);
        assert_eq!(term.nelen(), 2);
        assert_eq!(term.chlen(), 2);
        assert_eq!(term.nclen(), 1);
    }
}

// TODO: move space related method into the space module
// NOTE: commit 'f22c752' mentions fixing 'some bug/errors'
// amongst those was an object area validation bug which made valid areas not pass the check
// should have mentioned it by name in the commit message

#[cfg(test)]
mod container {}

#[cfg(test)]
mod text {}
