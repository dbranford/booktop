use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone, Default, Hash, Deserialize, Serialize)]
pub enum Read {
    Read,
    Reading,
    Stopped,
    #[default]
    Unread,
}

impl fmt::Display for Read {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Read::Read => write!(f, "Read"),
            Read::Reading => write!(f, "Reading"),
            Read::Stopped => write!(f, "Stopped"),
            Read::Unread => write!(f, "Unread"),
        }
    }
}

impl Read {
    pub const fn all() -> [Read; 4] {
        [Read::Read, Read::Reading, Read::Stopped, Read::Unread]
    }
    pub fn symbol(&self) -> char {
        match self {
            Read::Read => 'F',    //'📕',
            Read::Reading => 'R', //'📖',
            Read::Unread => 'U',  //'📚',
            Read::Stopped => 'S', //'🔖',
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub title: String,
    pub author: String,
    #[serde(default)]
    read: Read,
    #[serde(default)]
    pub tags: HashSet<String>,
}

impl Book {
    pub fn new(title: String, author: String) -> Book {
        Book {
            title,
            author,
            read: Read::Unread,
            tags: HashSet::new(),
        }
    }
    pub fn start(&mut self) {
        self.read = Read::Reading
    }
    pub fn finish(&mut self) {
        self.read = Read::Read
    }
    pub fn stop(&mut self) {
        self.read = Read::Stopped
    }
    pub fn reset(&mut self) {
        self.read = Read::Unread
    }
    pub fn read_state(&self) -> &Read {
        &self.read
    }
    pub fn tag(&mut self, tag: &str) -> bool {
        self.tags.insert(tag.to_string())
    }
    pub fn untag(&mut self, tag: &str) -> bool {
        self.tags.remove(tag)
    }
    pub fn contains_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

impl fmt::Display for Book {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}---{:?} ({})", self.title, self.author, self.read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    fn test_book() -> Book {
        Book {
            title: "Titular Title".to_string(),
            author: "Authoritative Author".to_string(),
            read: Read::Unread,
            tags: HashSet::new(),
        }
    }

    #[test]
    fn serialize() {
        let b = test_book();
        let s = serde_yaml::to_string(&b).unwrap();
        assert_eq!(
            s,
            "title: Titular Title\nauthor: Authoritative Author\nread: Unread\ntags: []\n"
                .to_string()
        );
    }

    #[test]
    fn deserialize() {
        let b = test_book();

        let r: Book =
            serde_yaml::from_str("title: Titular Title\nauthor: Authoritative Author\n").unwrap();
        assert_eq!(r, b);

        let r: Book = serde_yaml::from_str(
            "title: Titular Title\nauthor: Authoritative Author\nread: Unread\n",
        )
        .unwrap();
        assert_eq!(r, b);
    }
}
