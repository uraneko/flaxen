use crate::console::winsize::winsize;
use crate::render_pipeline;
use crate::space::{area_conflicts, between, border_fit, Border, Padding};
use crate::themes::Style;

use std::collections::{HashMap, HashSet};
use std::io::Error;
use std::io::StdoutLock;
use std::io::Write;

pub mod container;
pub mod term;
pub mod text;

// re-exports
pub use container::Container;
pub use term::Term;
pub use text::Text;

type TermTree = Vec<u8>;
type ContainerTree = Vec<[u8; 2]>;
type TextTree = Vec<[u8; 3]>;

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

impl Property {
    pub fn string(s: &str) -> Self {
        Self::String(s.to_string())
    }

    pub fn re_string(self, s: &str) -> Result<Self, Error> {
        if let Self::String(_) = self {
            return Ok(Self::String(s.to_string()));
        }

        Err(Error::other("variant is not a String"))
    }

    pub fn str(&mut self, str: &str) -> Result<(), Error> {
        if let Self::String(ref mut s) = self {
            s.push_str(str);

            return Ok(());
        }

        Err(Error::other("variant is not a String"))
    }

    pub fn char(&mut self, c: char) -> Result<(), Error> {
        if let Self::String(ref mut s) = self {
            s.push(c);

            return Ok(());
        }

        Err(Error::other("variant is not a String"))
    }

    pub fn find(&self, pat: &str) -> Result<Option<usize>, Error>
// NOTE: Pattern is unstable 
// where
    //     P: std::str::pattern::Pattern,
    {
        if let Self::String(ref s) = self {
            return Ok(s.find(pat));
        }

        Err(Error::other("variant is not a String"))
    }
}

impl Property {
    pub fn int(i: i64) -> Self {
        Self::Int(i)
    }

    pub fn re_int(self, i: i64) -> Result<Self, Error> {
        if let Self::Int(_) = self {
            return Ok(Self::Int(i));
        }

        Err(Error::other("variant is not a Int"))
    }

    pub fn int_add(&mut self, i: i64) -> Result<(), Error> {
        if let Self::Int(ref mut int) = self {
            *int += i;
            return Ok(());
        }
        Err(Error::other("variant is not a Int"))
    }

    pub fn int_sub(&mut self, i: i64) -> Result<(), Error> {
        if let Self::Int(ref mut int) = self {
            *int -= i;
            return Ok(());
        }
        Err(Error::other("variant is not a Int"))
    }

    pub fn int_mul(&mut self, i: i64) -> Result<(), Error> {
        if let Self::Int(ref mut int) = self {
            *int *= i;
            return Ok(());
        }
        Err(Error::other("variant is not a Int"))
    }

    pub fn int_div(&mut self, i: i64) -> Result<(), Error> {
        if let Self::Int(ref mut int) = self {
            *int /= i;
            return Ok(());
        }
        Err(Error::other("variant is not a Int"))
    }
}

impl Property {
    pub fn uint(u: u64) -> Self {
        Self::UInt(u)
    }

    pub fn re_uint(self, u: u64) -> Result<Self, Error> {
        if let Self::UInt(_) = self {
            return Ok(Self::UInt(u));
        }

        Err(Error::other("variant is not a UInt"))
    }

    pub fn uint_add(&mut self, u: u64) -> Result<(), Error> {
        if let Self::UInt(ref mut uint) = self {
            *uint += u;
            return Ok(());
        }
        Err(Error::other("variant is not a UInt"))
    }

    pub fn uint_sub(&mut self, u: u64) -> Result<(), Error> {
        if let Self::UInt(ref mut uint) = self {
            *uint -= u;
            return Ok(());
        }
        Err(Error::other("variant is not a UInt"))
    }

    pub fn uint_mul(&mut self, u: u64) -> Result<(), Error> {
        if let Self::UInt(ref mut uint) = self {
            *uint *= u;
            return Ok(());
        }
        Err(Error::other("variant is not a UInt"))
    }

    pub fn uint_div(&mut self, u: u64) -> Result<(), Error> {
        if let Self::UInt(ref mut uint) = self {
            *uint /= u;
            return Ok(());
        }
        Err(Error::other("variant is not a UInt"))
    }
}

impl Property {
    pub fn float(f: f64) -> Self {
        Self::Float(f)
    }

    pub fn re_float(self, f: f64) -> Result<Self, Error> {
        if let Self::Float(_) = self {
            return Ok(Self::Float(f));
        }

        Err(Error::other("variant is not a Float"))
    }

    pub fn float_add(&mut self, f: f64) -> Result<(), Error> {
        if let Self::Float(ref mut float) = self {
            *float += f;
            return Ok(());
        }
        Err(Error::other("variant is not a Float"))
    }

    pub fn float_sub(&mut self, f: f64) -> Result<(), Error> {
        if let Self::Float(ref mut float) = self {
            *float -= f;
            return Ok(());
        }
        Err(Error::other("variant is not a Float"))
    }

    pub fn float_mul(&mut self, f: f64) -> Result<(), Error> {
        if let Self::Float(ref mut float) = self {
            *float *= f;
            return Ok(());
        }
        Err(Error::other("variant is not a Float"))
    }

    pub fn float_div(&mut self, f: f64) -> Result<(), Error> {
        if let Self::Float(ref mut float) = self {
            *float /= f;
            return Ok(());
        }
        Err(Error::other("variant is not a Float"))
    }
}

impl Property {
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    pub fn re_bool(self, b: bool) -> Result<Self, Error> {
        if let Self::Bool(_) = self {
            return Ok(Self::Bool(b));
        }

        Err(Error::other("variant is not a Bool"))
    }

    pub fn negate(&mut self) {
        if let Self::Bool(ref mut b) = self {
            *b = !*b;
        }
    }
}

// Vec variant methods
impl Property {
    pub fn vec(vec: Vec<Property>) -> Self {
        Self::Vec(vec)
    }

    pub fn push(&mut self, p: Property) -> Result<(), Error> {
        if let Self::Vec(ref mut vec) = self {
            vec.push(p);
            return Ok(());
        }

        Err(Error::other("variant is not a vector"))
    }

    pub fn put(&mut self, p: Property, idx: usize) -> Result<(), Error> {
        if let Self::Vec(ref mut vec) = self {
            if idx > vec.len() {
                return Err(Error::other(
                    format!("index {} out of bounds", idx).as_str(),
                ));
            }
            vec.insert(idx, p);

            return Ok(());
        }

        Err(Error::other("variant is not a vector"))
    }

    pub fn pull(&mut self, idx: usize) -> Result<Property, Error> {
        if let Self::Vec(ref mut vec) = self {
            if idx > vec.len() - 1 {
                return Err(Error::other(
                    format!("index {} out of bounds", idx).as_str(),
                ));
            }

            return Ok(vec.remove(idx));
        }

        Err(Error::other("variant is not a vector"))
    }

    pub fn pop(&mut self) -> Result<Option<Property>, Error> {
        if let Self::Vec(ref mut vec) = self {
            return Ok(vec.pop());
        }

        Err(Error::other("variant is not a vector"))
    }
}

// shared methods between map and vec and string
impl Property {
    pub fn clear(&mut self) -> Result<(), Error> {
        if let Self::Vec(ref mut vec) = self {
            vec.clear();
            return Ok(());
        } else if let Self::Map(ref mut map) = self {
            map.clear();
            return Ok(());
        } else if let Self::String(ref mut s) = self {
            s.clear();
            return Ok(());
        }

        Err(Error::other("variant does not implement this method"))
    }

    pub fn len(&self) -> Result<usize, Error> {
        if let Self::Vec(ref vec) = self {
            return Ok(vec.len());
        } else if let Self::Map(ref map) = self {
            return Ok(map.len());
        } else if let Self::String(ref s) = self {
            return Ok(s.len());
        }

        Err(Error::other("variant does not implement this method"))
    }

    pub fn is_empty(&self) -> Result<bool, Error> {
        if let Self::Vec(ref vec) = self {
            return Ok(vec.is_empty());
        } else if let Self::Map(ref map) = self {
            return Ok(map.is_empty());
        } else if let Self::String(ref s) = self {
            return Ok(s.is_empty());
        }

        Err(Error::other("variant does not implement this method"))
    }
}

// Map variant methods
impl Property {
    pub fn map(map: HashMap<&'static str, Property>) -> Self {
        Self::Map(map)
    }

    pub fn insert(&mut self, k: &'static str, p: Property) -> Result<(), Error> {
        if let Self::Map(ref mut map) = self {
            map.insert(k, p);

            return Ok(());
        }

        Err(Error::other("variant is not a map"))
    }

    pub fn remove(&mut self, k: &str) -> Result<Option<Property>, Error> {
        if let Self::Map(ref mut map) = self {
            return Ok(map.remove(k));
        }

        Err(Error::other("variant is not a map"))
    }

    pub fn contains(&self, k: &str) -> Result<bool, Error> {
        if let Self::Map(ref map) = self {
            return Ok(map.contains_key(k));
        }

        Err(Error::other("variant is not a map"))
    }

    pub fn get(&self, k: &str) -> Result<Option<&Property>, Error> {
        if let Self::Map(ref map) = self {
            return Ok(map.get(k));
        }

        Err(Error::other("vaiant is not a map"))
    }

    pub fn get_or_insert(
        &mut self,
        k: &'static str,
        p: Property,
    ) -> Result<Option<&Property>, Error> {
        if let Ok(true) = self.contains(k) {
            self.get(k)
        } else {
            self.insert(k, p);

            Ok(None)
        }
    }
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

        let id = ne.id;

        term.push_container(c);
        term.push_nonedit(ne);

        let res = term.focus(&[5, 1, 8]);
        assert!(res.is_err());

        let res = term.focus(&id);
        assert!(res.is_ok());
        assert_eq!(term.on_focus.unwrap(), id);
        assert_eq!(term.focused().unwrap(), [11, 9]);
    }

    use crate::space::{Area, Border, Padding, Pos};

    #[test]
    fn cursor() {
        let mut term = Term::new(5);
        _ = term.container(
            &[0, 0],
            Pos::Value(56),
            Pos::Value(15),
            Area::Values { w: 35, h: 8 },
            Border::None,
            Padding::None,
        );
        _ = term.input(
            &[0, 0, 0],
            Pos::Value(1),
            Pos::Value(1),
            Area::Values { w: 23, h: 2 },
            Border::None,
            Padding::None,
        );

        let res = term.focus(&[0, 0, 0]);
        assert_eq!([term.cx, term.cy], [56 + 1 + 1, 15 + 1]);
    }

    #[test]
    fn objects() {
        let mut term = Term::new(0);
        term.push_container(Container::default());
        term.container(
            &[0, 1],
            Pos::Value(56),
            Pos::Value(15),
            Area::Values { w: 35, h: 18 },
            Border::None,
            Padding::None,
        );
        assert_eq!(term.containers.len(), 2);
        term.push_input({ Text::default() });
        term.nonedit(
            &[0, 1, 1],
            Pos::Value(12),
            Pos::Value(12),
            Area::Values { w: 2, h: 2 },
            Border::None,
            Padding::None,
            &[],
        );

        assert_eq!(term.tlen(), 2)
    }

    // test calc_text_abs_ori

    #[test]
    fn objects1() {
        let mut term = Term::new(0);
        term.push_container(Container::default());
        term.container(
            &[0, 1],
            Pos::Value(56),
            Pos::Value(15),
            Area::Values { w: 35, h: 18 },
            Border::None,
            Padding::None,
        );
        assert_eq!(term.containers.len(), 2);
        term.push_input({ Text::default() });
        term.nonedit(
            &[0, 1, 1],
            Pos::Value(12),
            Pos::Value(12),
            Area::Values { w: 2, h: 2 },
            Border::None,
            Padding::None,
            &[],
        );

        assert_eq!(term.tlen(), 2);
    }

    #[test]
    fn objects_count() {
        let mut term = Term::new(0);

        term.container(
            &[0, 0],
            Pos::Value(5),
            Pos::Value(5),
            Area::Values { w: 10, h: 10 },
            Border::None,
            Padding::None,
        );
        term.container(
            &[0, 1],
            Pos::Value(15),
            Pos::Value(15),
            Area::Values { w: 10, h: 10 },
            Border::None,
            Padding::None,
        );
        term.container(
            &[0, 2],
            Pos::Value(25),
            Pos::Value(25),
            Area::Values { w: 10, h: 10 },
            Border::None,
            Padding::None,
        );

        term.input(
            &[0, 2, 0],
            Pos::Value(1),
            Pos::Value(2),
            Area::Values { w: 2, h: 2 },
            Border::None,
            Padding::None,
        );
        term.nonedit(
            &[0, 1, 1],
            Pos::Value(2),
            Pos::Value(2),
            Area::Values { w: 2, h: 2 },
            Border::None,
            Padding::None,
            &[],
        );

        term.nonedit(
            &[0, 0, 1],
            Pos::Value(1),
            Pos::Value(1),
            Area::Values { w: 2, h: 2 },
            Border::None,
            Padding::None,
            &[],
        );

        assert_eq!(term.tlen(), 3);
        assert_eq!(term.ilen(), 1);
        assert_eq!(term.nelen(), 2);
        // assert_eq!(term.itlen(), 2);
        // assert_eq!(term.nitlen(), 1);
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
