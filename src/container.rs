// can have input, non editable or both
// so what are input or non editable?
// they are traits.
use std::collections::LinkedList;

use crate::Point;

#[derive(Debug)]
enum Text {
    Input(Input),
    NonEditable(NonEditable),
}

#[derive(Debug)]
struct Input {
    value: String,
    bounds: Point,
    origin: Point,
}

#[derive(Debug)]
struct NonEditable {
    value: String,
    bounds: Point,
    origin: Point,
}

#[derive(Debug)]
struct Container {
    items: LinkedList<Text>,
    bounds: Point,
    origin: Point,
}

impl std::fmt::Display for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_container() {
        let t0 = Text::Input(Input {
            value: "".to_string(),
            bounds: Point::new(3, 4),
            origin: Point::new(2, 4),
        });
        let t1 = Text::NonEditable(NonEditable {
            value: "".to_string(),
            bounds: Point::new(3, 4),
            origin: Point::new(2, 4),
        });

        let items: LinkedList<Text> = LinkedList::from([t0, t1]);

        let container = Container {
            items,
            bounds: Point::new(2, 3),
            origin: Point::new(2, 4),
        };

        println!("{:?}", container);
    }
}
