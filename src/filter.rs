use crate::book::{Book, Read};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Filter<'f> {
    pub author_match: Vec<&'f str>,
    pub read: HashSet<Read>,
}

impl<'f> Filter<'f> {
    fn match_book(self: &Self, book: &Book) -> bool {
        self.author_match
            .iter()
            .any(|a| string_match(a, &book.author))
            && self.read.contains(book.read_state())
    }
    pub fn filter_books<'b, T>(
        &'b self,
        books: Vec<(&'b T, &'b Book)>,
    ) -> impl Iterator<Item = (&T, &Book)> {
        books.into_iter().filter(|&(_, b)| self.match_book(b))
    }
}

fn string_match(s1: &str, s2: &str) -> bool {
    s1.eq_ignore_ascii_case(s2)
}
