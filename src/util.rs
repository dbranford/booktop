use crate::books::Bookcase;

pub fn example_bookcase() -> Bookcase {
    let mut books = Bookcase::new();
    books.add_book(
        "Great Expectations".to_string(),
        "Charles Dickens".to_string(),
    );
    books.add_book(
        "Journey to the Center of the Earth".to_string(),
        "Jules Verne".to_string(),
    );
    books
}
