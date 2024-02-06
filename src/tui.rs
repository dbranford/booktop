use crate::{
    book::{Book, Read},
    books::Bookcase,
    filter::Filter,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, List, ListState, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::{cmp::Ordering, io, iter::zip};

fn move_by(i: usize, δ: isize, l: usize) -> usize {
    match i.saturating_add_signed(δ) >= l {
        true => l - 1,
        false => i.saturating_add_signed(δ),
    }
}

#[derive(Debug)]
enum Popup {
    Filter,
}

struct App<'b> {
    bookcase: &'b Bookcase,
    popup: Option<Popup>,
    visible_books: Vec<usize>,
    state: TableState,
}

impl<'b> App<'b> {
    fn new(bookcase: &'b Bookcase) -> App {
        App {
            bookcase,
            popup: None,
            visible_books: bookcase.books.keys().cloned().collect(),
            state: TableState::default().with_selected(Some(0)),
        }
    }
    fn move_by(self: &mut Self, δ: isize) {
        if let Some(i) = self.state.selected() {
            let j = move_by(i, δ, self.visible_books.len());
            self.state.select(Some(j))
        }
    }
    fn move_to(self: &mut Self, i: isize) {
        match i.cmp(&0) {
            Ordering::Less => self
                .state
                .select(Some(self.visible_books.len().saturating_add_signed(i))),
            Ordering::Equal => self.state.select(Some(0)),
            Ordering::Greater => self.state.select(Some(
                usize::try_from(i - 1).expect("usize::try_from(i-1) on an i > 0 gated isize"),
            )),
        }
    }
    fn filter_currently_visible(self: &mut Self, filter: &Filter) {
        let matches = filter.filter_books(
            self.bookcase
                .books
                .iter()
                .filter(|(k, _)| self.visible_books.contains(k))
                .collect(),
        );
        self.visible_books = matches.map(|(&u, _)| u).collect()
    }
    fn reset_visible(self: &mut Self) {
        self.visible_books = self.bookcase.books.keys().cloned().collect()
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
        if let Some(p) = &app.popup {
            match p {
                Popup::Filter => {
                    let f = run_popup_filter(terminal, app.bookcase)?;
                    if let Some(f) = f {
                        app.filter_currently_visible(&f)
                    }
                }
            }
            app.popup = None
        }

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
                        Char('f') => app.popup = Some(Popup::Filter),
                        Char('F') => app.reset_visible(),
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
        .filter(|(u, _)| app.visible_books.contains(u))
        .map(|b| row_from_book(b));

    let contents = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Min(1),
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
        Cell::from(b.read_state().symbol().to_string()),
        Cell::from(b.title.as_str()),
        Cell::from(b.author.as_str()),
    ])
}

enum FilterPopupField {
    Author,
    Read,
}

impl FilterPopupField {
    fn next(&self) -> Self {
        use FilterPopupField::*;
        match self {
            Author => Read,
            Read => Author,
        }
    }
}

struct FilterPopupApp<'b> {
    authors: Vec<bool>,
    authors_state: ListState,
    author_list: Vec<&'b str>,
    read: [bool; Read::all().len()],
    read_state: ListState,
    current_field: FilterPopupField,
}

impl<'b> FilterPopupApp<'b> {
    fn new(books: &'b Bookcase) -> Self {
        let author_list = books.get_authors();
        let mut authors = Vec::new();
        authors.resize(author_list.len(), false);
        FilterPopupApp {
            authors,
            authors_state: ListState::default().with_selected(Some(0)),
            author_list,
            read: [false; Read::all().len()],
            read_state: ListState::default(),
            current_field: FilterPopupField::Author,
        }
    }
    fn move_by(&mut self, δ: isize) {
        match self.current_field {
            FilterPopupField::Author => {
                if let Some(i) = self.authors_state.selected() {
                    let j = move_by(i, δ, self.authors.len());
                    self.authors_state.select(Some(j))
                }
            }
            FilterPopupField::Read => {
                const READ_NO: usize = Read::all().len();
                if let Some(i) = self.read_state.selected() {
                    let j = move_by(i, δ, READ_NO);
                    self.read_state.select(Some(j))
                }
            }
        }
    }
    fn toggle(&mut self) {
        match self.current_field {
            FilterPopupField::Author => {
                if let Some(i) = self.authors_state.selected() {
                    self.authors[i] = !self.authors[i];
                }
            }
            FilterPopupField::Read => {
                if let Some(i) = self.read_state.selected() {
                    self.read[i] = !self.read[i];
                }
            }
        }
    }
    fn switch_fields(&mut self, new_field: FilterPopupField) {
        match self.current_field {
            FilterPopupField::Author => self.authors_state.select(None),
            FilterPopupField::Read => self.read_state.select(None),
        }
        self.current_field = new_field;
        match self.current_field {
            FilterPopupField::Author => self.authors_state.select(Some(0)),
            FilterPopupField::Read => self.read_state.select(Some(0)),
        }
    }
    fn tab(&mut self) {
        self.switch_fields(self.current_field.next())
    }
    fn to_filter(&self) -> Filter<'b> {
        Filter {
            author_match: zip(&self.authors, self.author_list.clone())
                .filter_map(|(b, a)| b.then_some(a))
                .collect(),
            read: zip(self.read, Read::all())
                .filter_map(|(b, r)| b.then_some(r))
                .collect(),
        }
    }
}

fn run_popup_filter<'f, B: Backend>(
    terminal: &mut Terminal<B>,
    books: &'f Bookcase,
) -> Result<Option<Filter<'f>>, io::Error> {
    let mut app_popup = FilterPopupApp::new(books);
    loop {
        terminal.draw(|rect| draw_popup_filter(rect, &mut app_popup))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Enter => return Ok(Some(app_popup.to_filter())),
                        Up => app_popup.move_by(-1),
                        Down => app_popup.move_by(1),
                        Esc => return Ok(None),
                        // Backspace | Delete => app_popup.backspace(),
                        Tab => app_popup.tab(),
                        Left | Right => app_popup.toggle(),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn selected_symbol(b: bool) -> &'static str {
    match b {
        true => "X",
        false => " ",
    }
}

fn draw_popup_filter(f: &mut Frame, app: &mut FilterPopupApp) {
    let area = popup_rect(80, 80, f.size());

    let popup_block = ratatui::widgets::Block::default().title("Apply filter");
    f.render_widget(popup_block, area);

    let highlight_style = Style::default().fg(Color::Yellow);

    let popup_filter_layout_vertical = Layout::new(
        Direction::Vertical,
        [Constraint::Min(1), Constraint::Length(4)],
    );
    let popup_filter_layout = popup_filter_layout_vertical.split(area);

    let author_block = Block::default().title("Author").borders(Borders::ALL);
    let author_list = List::new(app.author_list.iter().enumerate().map(|(i, a)| {
        let selected = selected_symbol(app.authors[i]);
        format!("[{selected}] {a}")
    }))
    .block(author_block)
    .highlight_style(highlight_style);
    f.render_stateful_widget(author_list, popup_filter_layout[0], &mut app.authors_state);

    let read_block = Block::default().title("Read").borders(Borders::ALL);
    let read_list = List::new(Read::all().iter().enumerate().map(|(i, r)| {
        let selected = selected_symbol(app.read[i]);
        format!("[{selected}] {r}")
    }))
    .block(read_block)
    .highlight_style(highlight_style);
    f.render_stateful_widget(read_list, popup_filter_layout[1], &mut app.read_state);
}

fn popup_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ],
    )
    .split(r);
    Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ],
    )
    .split(popup_layout[1])[1]
}
