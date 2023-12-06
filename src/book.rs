use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
enum Read {
    Read,
    Reading,
    Stopped,
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

impl Default for Read {
    fn default() -> Self {
        Read::Unread
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Book {
    pub title: String,
    author: String,
    #[serde(default)]
    read: Read,
}

impl Book {
    pub fn new(title: String, author: String) -> Book {
        Book {
            title,
            author,
            read: Read::Unread,
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
        }
    }

    #[test]
    fn serialize() {
        let b = test_book();
        let s = serde_yaml::to_string(&b).unwrap();
        assert_eq!(
            s,
            "title: Titular Title\nauthor: Authoritative Author\nread: Unread\n".to_string()
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
