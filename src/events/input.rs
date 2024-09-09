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

        self.border = crate::space::Border::Uniform('!');
    }

    pub fn homev(&mut self) {
        self.cy = 0
    }

    pub fn endv(&mut self) {
        self.cy = self.h - 1
    }

    // BUG: unicode characters take more space than one cell

    pub fn end(&mut self) {
        self.cx = self.w - 1;

        self.border = crate::space::Border::Uniform('+');
    }

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
mod test_input {
    use super::Text;

    #[test]
    fn test_put_char() {
        let mut i = Text::default();

        let mut idx = 0;
        ['p', 'i', 'k', 'a'].into_iter().for_each(|c| {
            i.put_char(c);
            idx += 1;

            assert_eq!(i.value[(i.cx - 1) as usize], Some(c));
            assert_eq!(idx, i.cx);
        })
    }

    #[test]
    fn test_backspace() {
        let mut i = Text::default();

        let input = "pikatchino";
        input.chars().into_iter().for_each(|c| i.put_char(c));

        i.delete();

        assert!({ i.cx as usize == input.len() - 1 && i.value[(i.cx - 1) as usize] == Some('n') });
    }

    #[test]
    fn test_to_end() {
        let mut i = Text::default();

        "pikatchaa".chars().into_iter().for_each(|c| i.put_char(c));
        // cursor is by default at end, but we still move it to end
        i.end();

        assert!({ i.cx == 9 && i.value[(i.cx - 1) as usize] == Some('a') });

        // now we test moving to end from somewhere else
        i.left();
        i.left();
        i.end();

        assert!({ i.cx == 9 && i.value[(i.cx - 1) as usize] == Some('a') });

        // and finally, moving to end from home (first cell in line)
        i.home();
        i.end();

        assert!({ i.cx == 9 && i.value[(i.cx - 1) as usize] == Some('a') });
    }

    #[test]
    fn test_to_home() {
        let mut i = Text::default();

        "pikatchuu".chars().into_iter().for_each(|c| i.put_char(c));
        i.home();

        assert!({ i.cx == 0 && i.value[(i.cx) as usize] == Some('p') });
    }

    #[test]
    fn test_to_the_right() {
        let mut i = Text::default();

        "pikatchau".chars().into_iter().for_each(|c| i.put_char(c));
        i.left();
        i.left();

        assert_eq!(i.value[(i.cx - 1) as usize], Some('h'));
        assert_eq!(i.cx as usize, "pikatchau".len() - 2);
    }

    #[test]
    fn test_to_the_left() {
        let mut i = Text::default();

        "pikatchau".chars().into_iter().for_each(|c| i.put_char(c));
        i.home();
        i.right();
        i.right();

        assert_eq!(i.value[(i.cx) as usize], Some('k'));
        assert_eq!(i.cx, 2);
    }

    #[test]
    fn test_cr_lf() {
        let mut i = Text::default();
        let mut user_input = String::new();

        "pikatcharu".chars().into_iter().for_each(|c| i.put_char(c));

        i.submit();

        // assert_eq!(
        // i.temp,
        //     "pikatcharu".chars().into_iter().collect::<Vec<char>>()
        // );
        assert!(i.value.is_empty());
        assert_eq!(i.cx, 0);
    }

    #[test]
    fn test_clear_line() {
        let mut i = Text::default();

        "pikauchi".chars().into_iter().for_each(|c| i.put_char(c));

        assert!({ i.cx as usize == "pikauchi".len() && i.value[(i.cx - 1) as usize] == Some('i') });

        // i.clear_line();
        assert!(i.value.is_empty());
        assert_eq!(i.cx, 0);
    }

    #[test]
    fn test_clear_right() {
        let mut i = Text::default();

        "pikatchiatto"
            .chars()
            .into_iter()
            .for_each(|c| i.put_char(c));
        (0..4).for_each(|_| {
            i.left();
        });

        // i.clear_right();
        assert_eq!(
            i.value.iter().map(|c| c.unwrap()).collect::<String>(),
            "pikatchi"
        );
    }

    #[test]
    fn test_clear_left() {
        let mut i = Text::default();

        "pikatchiatto"
            .chars()
            .into_iter()
            .for_each(|c| i.put_char(c));
        (0..4).for_each(|_| {
            i.left();
        });

        // i.clear_left();
        assert_eq!(
            i.value.iter().map(|c| c.unwrap()).collect::<String>(),
            "atto"
        );
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
