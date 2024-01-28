use crate::{book::Book, books::Bookcase};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
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
    fn move_by(self: &mut Self, δ: isize) {
        if let Some(i) = self.state.selected() {
            if i.saturating_add_signed(δ) >= self.bookcase.books.len() {
                self.state.select(Some(self.bookcase.books.len() - 1))
            } else {
                self.state.select(Some(i.saturating_add_signed(δ)))
            }
        }
    }
    fn move_to(self: &mut Self, i: isize) {
        match i {
            i if i < 0 => self
                .state
                .select(Some(self.bookcase.books.len().saturating_add_signed(i))),
            i if i == 0 => self.state.select(Some(0)),
            i if i > 0 => self.state.select(Some(usize::try_from(i - 1).unwrap())),
            _ => unreachable!(),
        }
    }
}

pub fn start_tui(books: &mut Bookcase) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(books);

    run_tui(&mut terminal, &mut app)?;

    disable_raw_mode()?;

    Ok(())
}

fn run_tui<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) -> Result<(), io::Error> {
    loop {
        terminal.draw(|rect| draw(rect, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Char('q') | Esc => return Ok(()),
                        Char('j') | Down => app.move_by(1),
                        Char('k') | Up => app.move_by(-1),
                        Char('G') => app.move_to(-1),
                        _ => {}
                    }
                }
            }
        }
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
        Cell::from(b.title.as_str()),
        Cell::from(b.author.as_str()),
    ])
}
