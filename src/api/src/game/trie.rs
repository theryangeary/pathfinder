use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    str::Chars,
};
use anyhow::Result;

#[derive(Debug, PartialEq, Clone)]
pub struct Trie {
    next: HashMap<char, Box<Trie>>,
    finish: bool,
}

impl Trie {
    fn new() -> Self {
        Trie {
            next: HashMap::new(),
            finish: false,
        }
    }

    fn insert(&mut self, word: &str) {
        match word.chars().next() {
            None => {
                self.finish = true;
                return;
            }
            Some(c) => {
                if self.next.get(&c).is_none() {
                    let trie = Trie::new();
                    self.next.insert(c, Box::new(trie));
                }
                if let Some(map) = self.next.get_mut(&c) {
                    map.insert(&word[1..]);
                }
            }
        }
    }

    fn isearch(&self, word: &mut Chars) -> bool {
        match word.next() {
            Some(c) => match self.next.get(&c) {
                Some(map) => map.isearch(word),
                None => false,
            },
            None => self.finish,
        }
    }

    pub fn search(&self, word: &str) -> bool {
        self.isearch(&mut word.chars())
    }

    fn ihas_prefix(&self, prefix: &mut Chars) -> bool {
        match prefix.next() {
            Some(c) => match self.next.get(&c) {
                Some(map) => map.ihas_prefix(prefix),
                None => false,
            },
            None => true, // Empty prefix always exists
        }
    }

    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.ihas_prefix(&mut prefix.chars())
    }
}

impl From<Vec<&str>> for Trie {
    fn from(words: Vec<&str>) -> Self {
        let mut result = Trie::new();
        for word in words {
            result.insert(word)
        }

        result
    }
}

impl From<PathBuf> for Trie {
    fn from(value: PathBuf) -> Self {
        let mut result = Trie::new();
        let file = File::open(value).expect("file does not exist");
        let buf = BufReader::new(file);
        buf.lines()
            .map(|l| l.expect("failed to parse line"))
            .for_each(|w| result.insert(&w));
        return result;
    }
}

impl Trie {
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let mut result = Trie::new();
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        for line in buf.lines() {
            let word = line?;
            result.insert(&word);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search() {
        let t = Trie::from(vec!["apple", "banana"]);
        assert!(t.search("apple"));
        assert!(t.search("banana"));
        assert!(!t.search("testingtesting123"));

        let t2 = Trie::from(vec!["apple", "app", "application", "applause", "happy"]);
        assert!(!t2.search("abdsas"));
        assert!(t2.search("happy"));
    }

    #[test]
    fn test_happy_in_wordlist() {
        let t = Trie::from(std::path::PathBuf::from("wordlist"));
        assert!(t.search("happy"), "The word 'happy' should be found in the wordlist");
    }

    #[test]
    fn test_has_prefix() {
        let t = Trie::from(vec!["apple", "app", "application", "applause", "happy"]);
        
        // Test valid prefixes
        assert!(t.has_prefix(""));
        assert!(t.has_prefix("a"));
        assert!(t.has_prefix("ap"));
        assert!(t.has_prefix("app"));
        assert!(t.has_prefix("appl"));
        assert!(t.has_prefix("appla"));
        assert!(t.has_prefix("ha"));
        assert!(t.has_prefix("hap"));
        assert!(t.has_prefix("happ"));
        
        // Test invalid prefixes
        assert!(!t.has_prefix("b"));
        assert!(!t.has_prefix("z"));
        assert!(!t.has_prefix("apple123"));
        assert!(!t.has_prefix("happyy"));
        assert!(!t.has_prefix("xyz"));
        
        // Test complete words (should also return true as they are valid prefixes)
        assert!(t.has_prefix("apple"));
        assert!(t.has_prefix("app"));
        assert!(t.has_prefix("happy"));
    }
}