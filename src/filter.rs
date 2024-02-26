use crate::book::{Book, Read};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug)]
pub struct Filter {
    pub author_match: Vec<Rc<str>>,
    pub read: HashSet<Read>,
    pub tags: Vec<String>,
}

impl Filter {
    fn match_book(&self, book: &Book) -> bool {
        (self.author_match.is_empty()
            || self
                .author_match
                .iter()
                .any(|a| string_match(a, &book.author)))
            && (self.read.is_empty() || self.read.contains(book.read_state()))
            && (self.tags.is_empty() || self.tags.iter().any(|t| book.tags.contains(t)))
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
