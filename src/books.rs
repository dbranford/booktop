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
    pub fn close<P: AsRef<Path>>(&self, path: P) {
        let _file = File::create(path).expect("Could not open file");
        serde_yaml::to_writer(_file, self).expect("Could not write to file");
    }
    pub fn add_book(&mut self, title: String, author: String) {
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
    pub fn get_books_by_keys<'k>(
        &'k self,
        keys: &'k [usize],
    ) -> impl Iterator<Item = (&'k usize, &'k Book)> {
        // Maybe this should return Item = Option<(&usize, &Book)> so that requested keys that do
        // not appear in self.books can be caught? Would require iterating through the keys and
        // using get_book. TUI should never send such a request, but CLI might?
        self.books.iter().filter(move |(k, _)| keys.contains(k))
    }
    pub fn get_authors(&self) -> Vec<&str> {
        let mut authors: Vec<&str> = self.books.values().map(|b| b.author.as_str()).collect();
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
    pub fn remove_book(&mut self, id: usize) {
        self.books.remove(&id);
    }
    pub fn util_renumber(&mut self) {
        let tmp = self.books.split_off(&0);
        for (ind, val) in tmp.into_values().enumerate() {
            self.books.insert(ind + 1, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_book1() -> Book {
        Book {
            title: "Titular Title".to_string(),
            author: "Authoritative Author".to_string(),
            tags: HashSet::from(["alpha".to_string(), "beta".to_string()]),
            ..Default::default()
        }
    }

    fn test_book2() -> Book {
        Book {
            title: "Uitular Title".to_string(),
            author: "Buthoritative Author".to_string(),
            tags: HashSet::from(["alpha".to_string(), "beta".to_string()]),
            ..Default::default()
        }
    }

    fn test_book3() -> Book {
        Book {
            title: "Vitular Title".to_string(),
            author: "Cuthoritative Author".to_string(),
            ..Default::default()
        }
    }

    fn test_bookcase() -> Bookcase {
        let b1 = test_book1();
        let b2 = test_book2();
        let b3 = test_book3();
        Bookcase {
            name: "Bookcase name".to_string(),
            books: BTreeMap::from([(1, b1), (2, b2), (3, b3)]),
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

    #[test]
    fn get_books() {
        let b = test_bookcase();

        assert_eq!(b.get_book(1), Some(&test_book1()));
        assert_eq!(b.get_book(9), None);

        let mut b = b;

        assert_eq!(b.get_mut_book(1), Some(&mut test_book1()));
        assert_eq!(b.get_mut_book(9), None);

        let b = b;

        // Do not care about order in which get_books_by_keys returns
        assert_eq!(
            HashMap::from_iter(b.get_books_by_keys(&[2, 1])),
            HashMap::from([(&1, &test_book1()), (&2, &test_book2())])
        );

        assert_eq!(
            b.get_books().into_iter().collect::<Vec<_>>(),
            vec![
                (&1, &test_book1()),
                (&2, &test_book2()),
                (&3, &test_book3())
            ]
        );
    }
}
