use crate::object_tree::{Term, Text};
use std::collections::HashMap;

use std::fs::{create_dir, metadata, File};
use std::io::Write;
use std::os::unix::fs::FileExt;

// TODO: make input cache that goes into cache files in the input dir
// TODO: make screenshots of an app state and cache them in the state dir

// serializes input entry into string
pub fn serialize_input(value: &[Option<char>]) -> String {
    value.iter().fold(String::new(), |mut acc, c| {
        if c.is_some() {
            acc.push(c.unwrap())
        } else {
            acc.push_str("\\_");
        }
        acc
    })
}

pub fn deserialize_input(value: &str) -> Vec<Option<char>> {
    let mut vec: Vec<Option<char>> = Vec::with_capacity(value.len());

    let mut value = value.chars();
    while let Some(v) = value.next() {
        if v == '\\' {
            match value.next() {
                Some('_') => vec.push(None),
                None => vec.push(Some('\\')),
                val => vec.extend(&[Some('\\'), val]),
            }
        } else {
            vec.push(Some(v))
        }
    }

    vec
}

impl Term {
    /// caches input history in the term cache map field
    pub fn cache_input(&mut self, entry: Vec<Option<char>>) {
        if !self.cache.contains_key("input") {
            self.cache.insert("input", vec![entry]);
        } else {
            self.cache.get_mut("input").unwrap().push(entry);
        }
    }

    /// loads input history from cached file
    /// if parent dir or cahce file do not exist
    /// this returns a None, otherwise it returns a Some()
    pub fn load_input(&self) -> Option<File> {
        if !std::path::Path::new("cache").is_dir() {
            return None;
        }

        std::fs::OpenOptions::new()
            .write(true)
            .open("cache/input.json")
            .ok()
        // .unwrap_or_else(|_| File::create("cache/input.json").unwrap())
    }

    /// caches the input history into its corresponding cache file
    /// if file already exists, this method appends to it
    /// else it creates a new file then fills it with the relevant cache contents
    pub fn save_input(&self, file: Option<File>) {
        if !std::path::Path::new("cache").is_dir() {
            create_dir("cache").unwrap();
            File::create("cache/input.json").unwrap();
        }

        let mut is_empty = false;
        let mut file = if file.is_some() {
            file.unwrap()
        } else {
            is_empty = true;

            File::create("cache/input.json").unwrap()
        };

        let cache: Vec<String> = self
            .cache
            .get("input")
            .unwrap()
            .iter()
            .map(|c| serialize_input(c))
            .collect::<Vec<String>>();

        match is_empty {
            false => {
                let mut s = format!("{:#?}}}", cache);
                s = s.replacen("\\\\", "\\", s.len() - 1);
                s.remove(0);

                let offset = metadata("cache/input.json").unwrap().len() - 3;

                file.write_at(s.as_bytes(), offset).unwrap();
            }
            true => {
                let mut s = format!("{{\"input\": {:#?}}}", cache);
                s = s.replacen("\\\\", "\\", s.len() - 1);

                file.write(s.as_bytes());
            }
        };
    }

    // fn save_screenshot(&self) {}
    // fn load_screenshot(&self) {}
}

impl Text {
    pub fn history_up(&mut self, cache: &[Vec<Option<char>>]) {
        if self.hicu == 0 {
            return;
        }

        if self.temp.is_empty() {
            self.temp = self.value.drain(..).collect();
        }

        self.value = cache[cache.len() - 1 - self.hicu].clone();
        self.hicu += 1;
    }

    pub fn history_down(&mut self, cache: &[Vec<Option<char>>]) {
        if self.hicu == cache.len() {
            return;
        } else if self.hicu == cache.len() - 1 {
            self.value = self.temp.drain(..).collect();
        }

        self.value = cache[cache.len() - 1 - self.hicu].clone();
        self.hicu -= 1;
    }

    // pub fn history_filter<'a>(
    //     &'a self,
    //     cache: &'a [Vec<Option<char>>],
    //     sidx: Option<usize>,
    // ) -> Option<(usize, &'a Vec<Option<char>>)> {
    //     cache
    //         .iter()
    //         .enumerate()
    //         .filter(|(idx, val)| {
    //             matching(val, &self.value)
    //                 && if sidx.is_some() {
    //                     idx < &sidx.unwrap()
    //                 } else {
    //                     true
    //                 }
    //         })
    //         .next()
    // }
}

// fn matching(cache: &[Option<char>], value: &[Option<char>]) -> bool {
//     for idx in 0..value.len() {
//         if value[idx] != cache[idx] {
//             return false;
//         }
//     }
//
//     true
// }

#[cfg(test)]
mod test_input {
    use std::collections::HashMap;

    #[test]
    fn serialize() {
        let value = &[
            Some('o'),
            None,
            Some('p'),
            Some('t'),
            Some(' '),
            Some('c'),
            None,
            Some('h'),
            Some('_'),
        ];

        let seria = String::from("o\\_pt c\\_h_");

        assert_eq!(seria, super::serialize_input(value));
    }

    #[test]
    fn deserialize() {
        let value = &vec![
            Some('o'),
            None,
            Some('p'),
            Some('t'),
            Some(' '),
            Some('c'),
            None,
            Some('h'),
            Some('_'),
        ];

        let seria = String::from("o\\_pt c\\_h_");

        assert_eq!(value, &super::deserialize_input(&seria));
    }

    #[test]
    fn cache() {
        let mut cache = HashMap::from([("input", vec![String::from("abc"), "123".to_string()])]);
        if cache.contains_key("input") {
            cache.get_mut("input").unwrap().push("def".to_string());
        }

        assert_eq!(
            cache.get("input"),
            Some(&vec![
                String::from("abc"),
                "123".to_string(),
                String::from("def")
            ])
        );

        let mut cache = HashMap::new();
        if !cache.contains_key("input") {
            cache.insert("input", vec!["098".to_string()]);
        }

        assert_eq!(cache.get("input"), Some(&vec!["098".to_string()]));
    }

    #[test]
    fn save() {}

    #[test]
    fn load() {}
}
