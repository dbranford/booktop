use crate::book::Book;
use clap::Args;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
enum BookArg {
    Id(usize),
    Title(String),
}

#[derive(Debug, Args)]
pub struct BookQuery {
    id: Option<usize>,

    #[clap(long)]
    title: Option<String>,
}

impl BookQuery {
    fn best_arg(self) -> BookArg {
        match (self.id, self.title) {
            (Some(id), Some(_title)) => {
                println!("Book id supplied, ignoring title");
                BookArg::Id(id)
            }
            (None, Some(title)) => BookArg::Title(title),
            (Some(id), None) => BookArg::Id(id),
            (None, None) => panic!("Specify a book"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn get_book(&self, query: BookQuery) -> Option<&Book> {
        match query.best_arg() {
            BookArg::Id(id) => self.books.get(&id),
            BookArg::Title(title) => match self.books.iter().find(|x| x.1.title == title) {
                Some(book_entry) => Some(book_entry.1),
                None => None,
            },
        }
    }
    pub fn get_mut_book(&mut self, query: BookQuery) -> Option<&mut Book> {
        match query.best_arg() {
            BookArg::Id(id) => self.books.get_mut(&id),
            BookArg::Title(title) => match self.books.iter_mut().find(|x| x.1.title == title) {
                Some(book_entry) => Some(book_entry.1),
                None => None,
            },
        }
    }
    pub fn get_books(&self) -> impl IntoIterator<Item = (&usize, &Book)> {
        &self.books
    }
    pub fn get_authors(&self) -> Vec<&str> {
        let mut authors: Vec<&str> = self.books.iter().map(|(_, b)| b.author.as_str()).collect();
        authors.dedup();
        authors
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
