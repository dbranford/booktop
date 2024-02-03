use crate::book::Book;

#[derive(Debug)]
pub struct Filter {
    pub author_match: Vec<String>,
}

impl Filter {
    fn match_book(self: &Self, book: &Book) -> bool {
        self.author_match
            .iter()
            .any(|a| string_match(a, &book.author))
    }
    pub fn filter_books<'b, T>(
        self: &'b Self,
        books: Vec<(&'b T, &'b Book)>,
    ) -> impl Iterator<Item = (&T, &Book)> {
        books.into_iter().filter(|&(_, b)| self.match_book(b))
    }
}

fn string_match(s1: &str, s2: &str) -> bool {
    s1.eq_ignore_ascii_case(s2)
}