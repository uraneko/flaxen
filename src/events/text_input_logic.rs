use crate::object_tree::Text;
use std::io::{StdoutLock, Write};

// inner logic of the input type of text objects
impl Text {
    // submit user input to the program
    pub fn submit(&mut self) -> Vec<Option<char>> {
        self.cx = 0;
        self.cy = 0;

        let v = self.value.clone();
        for val in &mut self.value {
            *val = None;
        }

        v
    }

    pub fn left(&mut self) {
        if self.cx == 0 && self.cy == 0 {
            return;
        }
        if self.cx == 0 && self.cy < self.h {
            self.cx = self.w - 1;
            self.cy -= 1;
        } else {
            self.cx -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.cx == self.w - 1 && self.cy == self.h - 1 {
            return;
        }

        if self.cx == self.w - 1 && self.cy < self.h {
            self.cx = 0;
            self.cy += 1;
        } else {
            self.cx += 1;
        }
    }

    pub fn up(&mut self) {
        if self.cy == 0 {
            return;
        }
        self.cy -= 1;
        // writer.write(b"\x1b[A");
    }

    pub fn down(&mut self) {
        if self.cy == self.h - 1 {
            return;
        }
        self.cy += 1;
    }

    pub fn home(&mut self) {
        self.cx = 0;
    }

    pub fn end(&mut self) {
        self.cx = self.w - 1;
    }

    pub fn homev(&mut self) {
        self.cy = 0
    }

    pub fn endv(&mut self) {
        self.cy = self.h - 1
    }

    // BUG: unicode characters take more space than one cell

    // put char if the input cursor points to non-empty (Some(c)) value in the value vec
    pub fn put_char(&mut self, c: char) {
        self.value
            .insert((self.cx + self.cy * self.w) as usize, Some(c));
        self.value.remove(self.value.len() - 1);

        if self.cx == self.w - 1 && self.cy == self.h - 1 {
            return;
        } else if self.cx == self.w - 1 {
            self.cx = 0;
            self.cy += 1;
        } else {
            self.cx += 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cx == 0 && self.cy == 0 {
            return;
        } else if self.cx == 0 {
            self.cx = self.w - 1;
            self.cy -= 1;
        } else {
            self.cx -= 1;
        }

        self.value.remove((self.cx + self.cy * self.w) as usize);
        self.value.push(None);

        // to delete line of value
        // position at self.w then call X on self.w
        // do for all lines then rewrite value
    }
}

#[cfg(test)]
mod input {
    use super::Text;

    #[test]
    fn submit() {
        let mut i = Text::default();
        i.w = 5;
        i.h = 1;
        i.value.resize((i.w * i.h) as usize, None);

        i.value[0] = Some('f');
        i.value[1] = Some('o');
        i.value[2] = Some('o');

        let res = i.submit();

        assert!(i.value.iter().all(|c| c.is_none()));
        assert_eq!(res, vec![Some('f'), Some('o'), Some('o'), None, None,]);
    }

    #[test]
    fn left() {
        let mut i = Text::default();
        i.w = 5;
        i.h = 2;
        i.value.resize((i.w * i.h) as usize, None);

        i.value[0] = Some('f');
        i.value[1] = Some('o');
        i.value[2] = Some('o');

        assert_eq!(i.cx, 0);
        assert_eq!(i.cy, 0);

        i.left();

        assert_eq!(i.cx, 0);
        assert_eq!(i.cy, 0);

        i.cy = 1;

        i.left();

        assert_eq!(i.cx, i.w - 1);
        assert_eq!(i.cy, 0);
    }

    // TODO: copy paste capabilities
    // TODO: render border polyform and borderless

    #[test]
    fn right() {
        let mut i = Text::default();
        i.w = 5;
        i.h = 2;
        i.value.resize((i.w * i.h) as usize, None);
        i.cx = i.w - 1;
        i.cy = i.h - 1;

        i.value[0] = Some('f');
        i.value[1] = Some('o');
        i.value[2] = Some('o');

        i.right();

        assert_eq!(i.cx, i.w - 1);
        assert_eq!(i.cy, i.h - 1);

        i.cy = 0;

        i.right();

        assert_eq!(i.cx, 0);
        assert_eq!(i.cy, 1);
    }

    #[test]
    fn up() {
        let mut i = Text::default();
        i.w = 3;
        i.h = 4;
        i.value.resize((i.w * i.h) as usize, None);
        i.cy = 3;

        i.up();
        assert_eq!(i.cy, 2);
        i.up();
        i.up();
        assert_eq!(i.cy, 0);
        i.up();
        assert_eq!(i.cy, 0);
    }

    #[test]
    fn down() {
        let mut i = Text::default();
        i.w = 3;
        i.h = 4;
        i.value.resize((i.w * i.h) as usize, None);
        i.cy = 0;

        i.down();
        assert_eq!(i.cy, 1);
        i.down();
        i.down();
        assert_eq!(i.cy, 3);
        i.down();
        assert_eq!(i.cy, 3);
    }

    #[test]
    fn home() {
        let mut i = Text::default();
        i.w = 6;
        i.h = 1;
        i.value.resize((i.w * i.h) as usize, None);
        i.cx = 3;

        i.home();
        assert_eq!(i.cx, 0);
    }

    #[test]
    fn end() {
        let mut i = Text::default();
        i.w = 6;
        i.h = 1;
        i.value.resize((i.w * i.h) as usize, None);
        i.cx = 4;

        i.end();
        assert_eq!(i.cx, i.w - 1);
    }

    #[test]
    fn homev() {
        let mut i = Text::default();
        i.w = 5;
        i.h = 4;
        i.value.resize((i.w * i.h) as usize, None);
        i.cy = 2;

        i.homev();
        assert_eq!(i.cy, 0);
    }

    #[test]
    fn endv() {
        let mut i = Text::default();
        i.w = 5;
        i.h = 4;
        i.value.resize((i.w * i.h) as usize, None);
        i.cy = 1;

        i.endv();
        assert_eq!(i.cy, i.h - 1);
    }
}

fn encode_char(c: char, bytes: &mut Vec<u8>) {
    match c.is_ascii() {
        false => bytes.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
        true => bytes.push(c as u8),
    }
}

fn str_to_bytes(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    s.chars()
        .into_iter()
        .for_each(|c| encode_char(c, &mut bytes));

    bytes
}
