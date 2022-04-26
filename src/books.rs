use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::path::PathBuf;

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

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Book {
    title: String,
    author: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookcase {
    name: String,
    pub books: BTreeMap<usize, Book>,
}

impl Bookcase {
    pub fn new() -> Bookcase {
        Bookcase {
            name: "Bookcase".to_string(),
            books: BTreeMap::new(),
        }
    }
    pub fn open(path: &PathBuf) -> Bookcase {
        let _file = File::open(path).expect("Could not open file");
        serde_json::from_reader(_file).expect("Couldn't extract bookcase")
    }
    pub fn close(&self, path: &PathBuf) -> () {
        let _file = File::create(path).expect("Could not open file");
        serde_json::to_writer(_file, self).expect("Could not write to file");
    }
    pub fn list(&mut self) -> () {
        println!("Bookcase: {}", self.name);
        println!("========================================");
        for (id, bk) in &self.books {
            println!("{}: {}", id, bk);
        }
    }
    pub fn add_book(&mut self, title: String, author: String) -> () {
        self.books.insert(
            self.books.keys().max().unwrap() + 1,
            Book::new(title, author),
        );
    }
    pub fn remove_book(&mut self, id: usize) -> () {
        self.books.remove(&id);
    }
    pub fn util_renumber(&mut self) -> () {
        let tmp = self.books.split_off(&0);
        for (ind, val) in tmp.into_values().enumerate() {
            self.books.insert(ind + 1, val);
        }
    }
}
