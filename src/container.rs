// can have input, non editable or both
// so what are input or non editable?
// they are traits.
use std::collections::HashMap;

pub type EventId = u8;

enum IdError {
    ProgramIsUnique,
}

enum IdKind {
    Text,
    Program,
    Container,
    EventsQueue,
}

// responsible for managing event queues and running events
pub struct Commissioner {}

impl Commissioner {
    fn new() -> Self {
        Self {}
    }

    fn authorize_id(&self, ik: IdKind) -> Result<u8, IdError> {
        match ik {
            IdKind::Text => Ok(0), // checks for self last TextId and returns the next one
            IdKind::EventsQueue => Ok(0),
            IdKind::Container => Ok(0),
            IdKind::Program => Err(IdError::ProgramIsUnique),
        }
    }

    fn approve_space(&self, edges: Edges) -> bool {
        true
    }
}

impl Commissioner {
    // start watching for events
    fn start(&self) {}

    // add events for observation
    fn extend(&self) {}

    // dont observe the event with the given id
    fn release(&self) {}

    // restart observing the event with the given id
    fn restore(&self) {}
}

use std::ops::Range;

#[derive(Debug)]
enum Text {
    Input(Input),
    NonEditable(NonEditable),
}

type ItemId = u8;
type ContainerId = u8;

#[derive(Debug)]
struct Input {
    id: ItemId,
    value: Vec<char>,
    edges: Edges,
    cursor: Range<usize>,
}

#[derive(Debug)]
struct NonEditable {
    id: ItemId,
    value: String,
    bounds: Range<usize>,
    origin: Range<usize>,
    cursor: Range<usize>,
}

struct InputInnerLogic;

impl Input {
    fn new(edges: Edges, id: TextId) -> Self {
        Self {
            value: vec![],
            edges,
            cursor: Range::default(),
            id,
        }
    }
}

type TextId = u8;

#[derive(Debug)]
pub struct Container {
    id: ContainerId,
    items: HashMap<TextId, Text>,
    edges: Edges,
    cursor: Range<usize>,
}

#[derive(Debug)]
pub(crate) struct Edges {
    top_right: Range<usize>,
    top_left: Range<usize>,
    bottom_right: Range<usize>,
    bottom_left: Range<usize>,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

// the Commissioner handles the ids evolution of all types with an id
// he also handles all events
// and he handles the space correctness of all types

impl Container {
    pub fn new(edges: Edges, id: u8) -> Self {
        Self {
            edges,
            id,
            items: Default::default(),
            cursor: Default::default(),
        }
    }

    fn resize() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_container() {
        let t0 = Text::Input(Input {
            value: vec![],
            bounds: Range::default(),
            origin: Range::default(),
            id: 0,
            cursor: Range::default(),
        });
        let t1 = Text::NonEditable(NonEditable {
            value: vec![],
            bounds: Range::default(),
            origin: Range::default(),
        });

        let items: LinkedList<Text> = LinkedList::from([t0, t1]);

        let container = Container {
            items,
            bounds: Range::default(),
            origin: Range::default(),
        };

        println!("{:?}", container);
    }
}
