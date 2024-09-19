use crate::console::winsize::winsize;
use crate::render_pipeline;
use crate::space::{area_conflicts, between, border_fit, Border, Padding};
use crate::themes::Style;

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::StdoutLock;
use std::io::Write;

pub mod container;
pub mod term;
pub mod text;

// re-exports
pub use container::Container;
pub use term::Term;
pub use text::Text;

type Styles = Vec<Style>;

#[derive(Debug)]
pub(crate) enum Property {
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
    // TODO: active should become a property
    active: u8,
    properties: HashMap<&'static str, Property>,
    w: u16,
    h: u16,
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
        let ws = winsize::from_ioctl();

        Self {
            terms: vec![Term::new(0)],
            active: 0,
            w: ws.cols(),
            h: ws.rows(),
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

#[cfg(test)]
mod tree {
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
mod test_term {
    use super::{Container, Term, Text};

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
mod test_container {}

#[cfg(test)]
mod test_text {}
