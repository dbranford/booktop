use crate::book::Book;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Bookcase {
    pub name: String,
    pub books: BTreeMap<usize, Book>,
}

impl Bookcase {
    pub fn new() -> Bookcase {
        Bookcase {
            name: "Bookcase".to_string(),
            books: BTreeMap::new(),
        }
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Bookcase {
        let _file = File::open(path).expect("Could not open file");
        serde_yaml::from_reader(_file).expect("Couldn't extract bookcase")
    }
    pub fn close<P: AsRef<Path>>(&self, path: P) -> () {
        let _file = File::create(path).expect("Could not open file");
        serde_yaml::to_writer(_file, self).expect("Could not write to file");
    }
    pub fn add_book(&mut self, title: String, author: String) -> () {
        let key = match self.books.keys().max() {
            Some(max_key) => max_key + 1,
            None => 1,
        };
        self.books.insert(key, Book::new(title, author));
    }
    pub fn get_book(&self, id: usize) -> Option<&Book> {
        self.books.get(&id)
    }
    pub fn get_mut_book(&mut self, id: usize) -> Option<&mut Book> {
        self.books.get_mut(&id)
    }
    pub fn get_books(&self) -> impl IntoIterator<Item = (&usize, &Book)> {
        &self.books
    }
    pub fn get_authors(&self) -> Vec<&str> {
        let mut authors: Vec<&str> = self.books.iter().map(|(_, b)| b.author.as_str()).collect();
        authors.dedup();
        authors
    }
    pub fn get_tags(&self) -> Vec<String> {
        let mut tags: HashSet<String> = HashSet::new();
        for b in self.books.values() {
            tags.extend(b.tags.clone())
        }
        tags.into_iter().collect()
    }
    pub fn pick_book(&self) -> (&usize, &Book) {
        let mut rng = rand::thread_rng();
        match self.books.iter().choose(&mut rng) {
            Some(book) => book,
            None => panic!("No books to pick from"),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bookcase() -> Bookcase {
        let mut b1 = Book::new(
            "Titular Title".to_string(),
            "Authoritative Author".to_string(),
        );
        let mut b2 = Book::new(
            "Uitular Title".to_string(),
            "Buthoritative Author".to_string(),
        );
        b1.tag("alpha");
        b1.tag("beta");
        b2.tag("alpha");
        Bookcase {
            name: "Bookcase name".to_string(),
            books: BTreeMap::from([(1, b1), (2, b2)]),
        }
    }

    #[test]
    fn get_tags() {
        let b = test_bookcase();
        let tags = b.get_tags();
        let mut tags_dedup = tags.clone();
        tags_dedup.dedup();

        // Vec has no duplicates
        assert_eq!(tags, tags_dedup);
        // Vec has correct contents in any order
        assert_eq!(
            HashSet::<String>::from_iter(tags),
            HashSet::<String>::from_iter(vec!["alpha".to_string(), "beta".to_string()])
        )
    }
}
