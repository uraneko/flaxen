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
            .open(format!("cache/input/{}.json", key));

        // BUG???: unwrap_or should unwrap if some or return None and exit the entire method if
        // none
        // instead it jumps to returning None even when f is actually Some(File)
        let mut f = match f {
            Ok(f) => f,
            Err(e) => return None,
        };

        let mut cache = String::new();
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
mod json {
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
}

#[cfg(test)]
mod input_cache {
    use std::path::Path;

    use super::Term;

    #[test]
    fn cache() {
        let mut term = Term::new(0);

        let cache = vec![
            vec![Some('f'), None, Some('o')],
            vec![Some('b'), None, None, Some('a')],
            vec![Some('b'), Some('a'), None, Some('z')],
        ];

        cache
            .iter()
            .for_each(|c| term.cache_input("inputtest", c.clone()));

        assert!(term.cache.contains_key("inputtest"));
        assert_eq!(&cache, term.cache.get("inputtest").unwrap());
    }

    // NOTE: this test works if ran alone
    // but if ran together with the others in this file it fails the first time then keeps passing
    // after that
    #[test]
    fn load() {
        let mut term = Term::new(0);

        let cache = vec![
            vec![Some('f'), None, Some('o')],
            vec![Some('b'), None, None, Some('a')],
            vec![Some('b'), Some('a'), None, Some('z')],
        ];

        cache
            .iter()
            .for_each(|c| term.cache_input("inputtest", c.clone()));

        term.save_input("inputtest", None);

        term.cache.get_mut("inputtest").unwrap().clear();

        let cache_file = term.load_input("inputtest");

        assert_eq!(&cache, term.cache.get("inputtest").unwrap());
    }

    // NOTE: this test fails arbitrarily too
    #[test]
    fn save() {
        std::fs::remove_file("cache/input/inputtest.json").unwrap_or(());

        let mut term = Term::new(0);

        let cache = vec![
            vec![Some('f'), None, Some('o')],
            vec![Some('b'), None, None, Some('a')],
            vec![Some('b'), Some('a'), None, Some('z')],
        ];

        cache
            .iter()
            .for_each(|c| term.cache_input("inputtest", c.clone()));

        let cache_file = term.load_input("inputtest");

        assert!(cache_file.is_none());

        term.save_input("inputtest", cache_file);

        assert!(Path::new("cache/input").is_dir());
        assert!(Path::new("cache/input/inputtest.json").is_file());
    }
}

#[cfg(test)]
mod text_history {
    use super::{Term, Text};

    #[test]
    fn history_up() {
        let mut text = Text::default();
        text.name = "inputtest".to_string();
        text.w = 5;
        text.h = 1;
        text.value.resize((text.w * text.h) as usize, None);

        let cache = vec![
            vec![Some('f'), None, Some('o')],
            vec![Some('b'), None, None, Some('a')],
            vec![Some('b'), Some('a'), None, Some('z')],
        ];

        text.history_up(&cache);

        assert_eq!(text.value[..4], cache[2]);
    }

    #[test]
    fn history_down() {
        let mut text = Text::default();
        text.name = "inputtest".to_string();
        text.w = 5;
        text.h = 1;
        text.value.resize((text.w * text.h) as usize, None);
        text.hicu = 3;

        let cache = vec![
            vec![Some('f'), None, Some('o')],
            vec![Some('b'), None, None, Some('a')],
            vec![Some('b'), Some('a'), None, Some('z')],
        ];

        text.history_down(&cache);
        assert_eq!(&text.value[..4], &cache[1]);

        let temp = vec![Some('!'), Some('@'), Some('#'), Some('$'), Some('%')];
        text.temp = temp.clone();

        text.hicu = 1;
        text.history_down(&cache);

        assert!(text.temp.is_empty());
        assert_eq!(text.value, temp);
    }
}
