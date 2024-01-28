use crate::book::Book;
use crate::books::Bookcase;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use ratatui::{Frame, Terminal};
use std::io;

pub fn start_tui(books: &Bookcase) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|rect| draw(rect, books))?;
    }
}

fn draw(rect: &mut Frame, books: &Bookcase) {
    let size = rect.size();
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(1), Constraint::Min(2)],
    )
    .split(size);

    let title = Paragraph::new("Booktop");

    let rows = books.get_books().into_iter().map(|b| row_from_book(b));

    let contents = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(30),
            Constraint::Length(30),
        ],
    );

    rect.render_widget(title, chunks[0]);
    rect.render_widget(contents, chunks[1]);
}

fn row_from_book<'b>((i, b): (&'b usize, &'b Book)) -> Row {
    Row::new(vec![
        Cell::from(i.to_string()),
        Cell::from(b.title.clone()),
        Cell::from(b.author.clone()),
    ])
}
