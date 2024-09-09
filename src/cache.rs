use crate::object_tree::{Term, Text};

use std::collections::HashMap;
use std::fs::{create_dir, create_dir_all, metadata, File};
use std::io::Read;
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
    pub fn cache_input(&mut self, key: &str, entry: Vec<Option<char>>) {
        if entry.iter().all(|c| c.is_none()) {
            return;
        } else if !self.cache.contains_key(key) {
            self.cache.insert(key.to_owned(), vec![entry]);
        } else {
            self.cache.get_mut(key).unwrap().push(entry);
        }
    }

    /// loads input history from cached file
    /// if parent dir or cahce file do not exist
    /// this returns a None, otherwise it returns a Some()
    pub fn load_input(&mut self, key: &str) -> Option<File> {
        if !std::path::Path::new("cache/input").is_dir() {
            return None;
        }

        let f = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(format!("cache/input/{}.json", key))
            .ok();

        let mut cache = String::new();
        let mut f = f.unwrap();

        f.read_to_string(&mut cache).unwrap();

        let cache = cache
            .trim_start_matches("{\"cache\": [")
            .trim_end_matches("]}");

        let mut lines = cache.lines().filter(|l| !l.is_empty());

        let cache = match self.cache.get_mut(key) {
            None => {
                self.cache.insert(key.to_string(), vec![]);
                self.cache.get_mut(key).unwrap()
            }
            Some(cache) => cache,
        };

        while let Some(mut line) = lines.next() {
            line = line.trim().trim_start_matches('"').trim_end_matches("\",");
            cache.push(deserialize_input(&line));
            // print!("{{ {} }}", line);
        }

        // println!("{:?}", self.cache);

        Some(f)
    }

    /// caches the input history into its corresponding cache file
    /// if file already exists, this method appends to it
    /// else it creates a new file then fills it with the relevant cache contents
    pub fn save_input(&self, key: &str, file: Option<File>) {
        if !std::path::Path::new("cache/input").is_dir() {
            create_dir_all("cache/input").unwrap();
            File::create(format!("cache/input/{}.json", key)).unwrap();
        }

        let mut is_empty = false;
        let mut file = if file.is_some() {
            file.unwrap()
        } else {
            is_empty = true;

            File::create(format!("cache/input/{}.json", key)).unwrap()
        };

        let cache: Vec<String> = self
            .cache
            .get(key)
            .unwrap()
            .iter()
            .map(|c| serialize_input(c))
            .collect::<Vec<String>>();

        match is_empty {
            false => {
                let mut s = format!("{:#?}}}", cache);
                s = s.replacen("\\\\", "\\", s.len() - 1);
                s.remove(0);

                let offset = metadata(format!("cache/input/{}.json", key)).unwrap().len() - 3;

                file.write_at(s.as_bytes(), offset).unwrap();
            }
            true => {
                let mut s = format!("{{\"cache\": {:#?}}}", cache);
                s = s.replacen("\\\\", "\\", s.len() - 1);

                file.write(s.as_bytes());
            }
        };
    }

    // fn save_screenshot(&self) {}
    // fn load_screenshot(&self) {}
}

impl Text {
    // cases:
    // hicu = 0 -> is cache empty? yes return : no carry on
    // hicu = cache.len() -> reached end of cache, return
    // hicu > 0 and hicu < cache.len() -> carry on
    pub fn history_up(&mut self, cache: &[Vec<Option<char>>]) {
        // print!("\r\n\n\n\n\n\n\nin: {}, {}", self.hicu, cache.len());
        if (self.hicu == 0 && cache.is_empty()) || self.hicu == cache.len() {
            return;
        } else if (self.hicu == 0 && !cache.is_empty())
            || (self.hicu > 0 && self.hicu < cache.len())
        {
            if self.hicu == 0 && !cache.is_empty() {
                // update temp and clear value
                self.temp = self.value.clone();
            }
            self.value.iter_mut().for_each(|v| *v = None);

            // always increment hicu before using it in cache index
            self.hicu += 1;

            // get cached value and clone it to self.value
            let value = &cache[cache.len() - self.hicu];

            // NOTE: this module does nothing to validate this assertion at any point
            assert!(self.value.len() >= value.len());

            for idx in 0..value.len() {
                self.value[idx] = value[idx];
            }
        }
        // print!("\r\n\n\n\n\n\n\nout: {:?}", self.hicu);
    }

    // cases:
    // hicu = 0 -> return
    // hicu = 1 -> go to 0, return temp to value
    // hicu > 0 && hicu < cache.len -> carry on
    pub fn history_down(&mut self, cache: &[Vec<Option<char>>]) {
        // print!("\r\n\n\n\n\n\n\nin: {:?}", self.hicu);
        if self.hicu == 0 {
            return;
        } else if self.hicu == 1 {
            self.hicu -= 1;
            assert_eq!(self.value.len(), self.temp.len());
            self.value = self.temp.drain(..).collect();
        } else if self.hicu > 0 && self.hicu <= cache.len() {
            self.hicu -= 1;

            self.value.iter_mut().for_each(|v| *v = None);

            let value = &cache[cache.len() - self.hicu];

            // NOTE: this module does nothing to validate this assertion at any point
            assert!(self.value.len() >= value.len());

            for idx in 0..value.len() {
                self.value[idx] = value[idx];
            }
        }

        // print!("\r\n\n\n\n\n\n\n\nout: {:?}", self.hicu);
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
