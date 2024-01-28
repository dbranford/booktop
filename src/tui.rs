use crate::book::Book;
use crate::books::Bookcase;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Paragraph, Row, Table, TableState};
use ratatui::{Frame, Terminal};
use std::io;

struct App<'b> {
    bookcase: &'b Bookcase,
    state: TableState,
}

impl<'b> App<'b> {
    fn new(bookcase: &'b Bookcase) -> App {
        App {
            bookcase,
            state: TableState::default().with_selected(Some(0)),
        }
    }
}

pub fn start_tui(books: &mut Bookcase) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(books);

    terminal.clear()?;

    loop {
        terminal.draw(|rect| draw(rect, &mut app))?;
    }
}

fn draw(rect: &mut Frame, app: &mut App) {
    let size = rect.size();
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Length(1), Constraint::Min(2)],
    )
    .split(size);

    let highlight_style = Style::default().fg(Color::Yellow);

    let title = Paragraph::new("Booktop");

    let rows = app
        .bookcase
        .get_books()
        .into_iter()
        .map(|b| row_from_book(b));

    let contents = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(30),
            Constraint::Length(30),
        ],
    )
    .highlight_style(highlight_style);

    rect.render_widget(title, chunks[0]);
    rect.render_stateful_widget(contents, chunks[1], &mut app.state);
}

fn row_from_book<'b>((i, b): (&'b usize, &'b Book)) -> Row {
    Row::new(vec![
        Cell::from(i.to_string()),
        Cell::from(b.title.clone()),
        Cell::from(b.author.clone()),
    ])
}
