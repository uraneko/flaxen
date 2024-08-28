use crate::space_awareness::{Border, Padding, Point, SpaceAwareness};
use crate::styles::StyleStrategy;
use crate::termbuf::winsize;

use std::any::type_name;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Zero;

#[derive(Debug)]
pub struct ObjectTree {
    terms: Vec<Term>,
}

#[derive(Debug)]
pub enum TreeErrors {
    BadID,
    IDExists,
    ParentNotFound,
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

    pub fn container(&mut self, id: &[u8]) -> Result<(), TreeErrors> {
        if id.len() > 2 || !self.has_term(id[0]) || self.has_container(&[id[0], id[1]]) {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        self.term_ref_mut(id[0])
            .unwrap()
            .containers
            .push(Container::new([id[0], id[1]]));

        Ok(())
    }

    /// takes only term id and automatically assigns an id for the container
    /// returns the full new container id
    pub fn container_auto(&mut self, id: u8) -> Result<[u8; 2], TreeErrors> {
        /// this should actually fail
        if !self.has_term(id) {
            return Err(TreeErrors::ParentNotFound);
        }

        let id = [id, self.assign_container_id(id)];

        self.term_ref_mut(id[0])
            .unwrap()
            .containers
            .push(Container::new(id));

        Ok(id)
    }

    pub fn input(&mut self, id: &[u8]) -> Result<(), TreeErrors> {
        if id.len() > 3
            || id[2] % 2 != 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_input(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        self.container_ref_mut(&[id[0], id[1]])
            .unwrap()
            .items
            .push(Text::new([id[0], id[1], id[2]]));

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the input
    /// returns the full new input id
    pub fn input_auto(&mut self, id: &[u8]) -> Result<[u8; 3], TreeErrors> {
        if id.len() > 2 {
            eprintln!("use self.input(id) instead");
            return Err(TreeErrors::BadID);
        }

        if !self.has_container(&[id[0], id[1]]) {
            eprintln!("bad id");
            return Err(TreeErrors::ParentNotFound);
        }

        let id = [id[0], id[1], self.assign_input_id(id[0], id[1])];

        self.container_ref_mut(&[id[0], id[1]])
            .unwrap()
            .items
            .push(Text::new(id));

        Ok(id)
    }

    pub fn nonedit(&mut self, id: &[u8]) -> Result<(), TreeErrors> {
        if id.len() > 3
            || id[2] % 2 == 0
            || !self.has_container(&[id[0], id[1]])
            || self.has_nonedit(&[id[0], id[1], id[2]])
        {
            eprintln!("bad id");
            return Err(TreeErrors::BadID);
        }

        self.container_ref_mut(&[id[0], id[1]])
            .unwrap()
            .items
            .push(Text::new([id[0], id[1], id[2]]));

        Ok(())
    }

    /// takes only term and container ids and automatically assigns an id for the nonedit
    /// returns the full new nonedit id
    pub fn nonedit_auto(&mut self, id: &[u8]) -> Result<[u8; 3], TreeErrors> {
        if id.len() > 2 {
            eprintln!("use self.nonedit(id) instead");
            return Err(TreeErrors::BadID);
        }

        if !self.has_container(&[id[0], id[1]]) {
            eprintln!("bad id");
            return Err(TreeErrors::ParentNotFound);
        }

        let id = [id[0], id[1], self.assign_nonedit_id(id[0], id[1])];

        self.container_ref_mut(&[id[0], id[1]])
            .unwrap()
            .items
            .push(Text::new(id));

        Ok(id)
    }

    /// returns an optional immutable reference of the term with the provided id if it exists
    pub fn term_ref(&self, id: u8) -> Option<&Term> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// returns an optional mutable reference of the term with the provided id if it exists
    pub fn term_ref_mut(&mut self, id: u8) -> Option<&mut Term> {
        self.terms.iter_mut().find(|t| t.id == id)
    }

    /// returns an optional immutable reference of the container with the provided id if it exists
    pub fn container_ref(&self, id: &[u8; 2]) -> Option<&Container> {
        let Some(term) = self.term_ref(id[0]) else {
            return None;
        };

        term.containers.iter().find(|c| &c.id == id)
    }

    /// returns an optional mutable reference of the container with the provided id if it exists
    pub fn container_ref_mut(&mut self, id: &[u8; 2]) -> Option<&mut Container> {
        let Some(term) = self.term_ref_mut(id[0]) else {
            return None;
        };

        term.containers.iter_mut().find(|c| &c.id == id)
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

    // methods of the has_object series do not check for duplicate ids
    // because those are already being screened by earlier id assignment methods
    // and there is no way in the api to bypass those checks and push an object to the tree
    // which means that duplicate ids can never happen
    pub fn has_term(&self, term: u8) -> bool {
        self.terms.iter().find(|t| t.id == term).is_some()
    }

    pub fn has_container(&self, id: &[u8; 2]) -> bool {
        match self.term_ref(id[0]) {
            Some(term) => term.containers.iter().find(|c| c.id == *id).is_some(),
            None => {
                eprintln!("no term with such id: {}", id[0]);
                false
            }
        }
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

    pub fn assign_container_id(&self, term: u8) -> u8 {
        // NOTE: this method should always be called inside another method/fn
        // that checks before calling this method that
        // the parent term with the given id exists
        let term = self.term_ref(term).unwrap();
        let mut id = 0;
        for cont in &term.containers {
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

#[derive(Debug)]
pub struct Term {
    pub overlay: bool,
    pub id: u8,
    pub cache: HashMap<&'static str, Vec<u8>>,
    pub buf: Vec<char>,
    pub width: u16,
    pub height: u16,
    pub cursor: Point<u16>,
    pub containers: Vec<Container>,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
}

impl Term {
    pub fn new(id: u8) -> Term {
        let ws = winsize::from_ioctl();

        let mut buf = vec![];
        buf.resize((ws.rows() * ws.cols()) as usize, ' ');

        Term {
            id,
            overlay: false,
            cache: HashMap::new(),
            cursor: Point::new(0, 0),
            width: ws.cols(),
            height: ws.rows(),
            buf,
            containers: vec![],
            registry: HashSet::from(["Zero"]),
            padding: Padding::None,
            border: Border::Some('?'),
        }
    }

    // prints the buffer, respecting width and height
    pub fn print_buf(&self) {
        for idxr in 0..self.buf.len() / self.width as usize {
            print!("\r\n");
            for idxc in 0..self.buf.len() / self.height as usize {
                print!("{}", self.buf[idxr]);
            }
        }
    }

    /// places the cursor at the new position
    pub fn place(&mut self, x: u16, y: u16) {
        let esc_seq = format!("\x1b{};{}f", x, y);
        self.cursor.place(x, y);
    }

    // TODO: partial clear/render

    /// rewrites the buffer according to new spaces, positions and events
    pub fn process(&mut self) {
        self.containers.iter().for_each(|c| {
            let buf = c.buffer();
            let [mut x, mut y] = [0, 0];
            let [xmin, ymin] = [c.origin.x(), c.origin.y()];
            let [xmax, ymax] = [c.origin.x() + c.width, c.origin.y() + c.height];
            while y < ymax {
                while x < xmax {
                    let tpt = xmin + ymin + x + y * c.width;
                    let bpt = x + y * c.width;
                    self.buf[tpt as usize] = buf[bpt as usize];
                    x += 1;
                }
                x = 0;
                y += 1;
            }
        })
    }

    /// renders the whole buffer into the terminal
    /// use after clear
    pub fn render(&self, writer: &mut StdoutLock) {
        assert_eq!(self.buf.len() as u16, self.width * self.height);
        let mut line_break = self.width;
        self.buf.iter().for_each(|cell| {
            // chnage buffer items to be chars
            writer.write(format!("{}", cell).as_bytes());
            line_break -= 1;
            if line_break == 0 {
                writer.write(&[13, 10]);
                line_break = self.width;
            }
        });
    }

    /// clears the whole terminal display
    pub fn clear(&self, writer: &mut StdoutLock) {
        writer.write(b"\x1b[2J");
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
}

// TODO: need a way

#[derive(Debug)]
pub struct Container {
    pub overlayed: bool,
    pub id: [u8; 2],
    pub items: Vec<Text>,
    pub width: u16,
    pub height: u16,
    pub origin: Point<u16>,
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

impl Container {
    pub fn new(id: [u8; 2]) -> Container {
        Container {
            id,
            overlayed: false,
            registry: HashSet::new(),
            width: 37,
            height: 5,
            origin: Point::<u16>::new(17, 0),
            items: vec![],
            padding: Padding::None,
            border: Border::Some('?'),
        }
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
        let line_break = self.width;
        let mut buf = Vec::with_capacity((self.width * self.height) as usize);
        self.items.iter().for_each(|i| {
            // write the outer padding
            // write the borders
            // write the inner padding
            // write the value
            let mut x = 0;
            let mut y = 0;
            let (xmin, xmax) = (i.origin.x(), i.origin.x() + i.width);
            let (ymin, ymax) = (i.origin.y(), i.origin.y() + i.height);
            let line_break = xmax;
            while y < ymin {
                y += 1;
                buf.extend(
                    (0..self.width)
                        .into_iter()
                        .map(|u| ' ')
                        .collect::<Vec<char>>(),
                )
            }

            while x < xmin {
                x += 1;
                buf.push(' ')
            }

            buf.extend(i.value.clone());
            while x < xmax {
                x += 1;
                buf.push(' ');
            }

            while y < ymax {
                y += 1;
                buf.extend(
                    (0..self.width)
                        .into_iter()
                        .map(|u| ' ')
                        .collect::<Vec<char>>(),
                )
            }
        });

        buf
    }
}

#[derive(Debug)]
pub struct Text {
    pub overlayed: bool,
    edit: bool,
    pub id: [u8; 3],
    pub value: Vec<char>,
    pub width: u16,
    pub height: u16,
    pub cursor: Point<u16>,
    pub origin: Point<u16>,
    pub registry: HashSet<&'static str>,
    pub border: Border,
    pub padding: Padding,
}

// Inputs can only have pair IDs
// while NonEdits can only have odd IDs
impl Text {
    pub fn new(id: [u8; 3]) -> Text {
        Text {
            id,
            edit: true,
            width: 37,
            height: 5,
            origin: Point::<u16>::new(6, 0),
            cursor: Point::<u16>::new(0, 0),
            overlayed: false,
            value: vec!['h', 'e', 'l', 'l', 'o', ' ', 't', 'e', 'r', 'm'],
            registry: HashSet::new(),
            padding: Padding::None,
            border: Border::Some('?'),
        }
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
