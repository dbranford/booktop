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
    widgets::{Block, Cell, List, ListState, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::{cmp::Ordering, fmt::Display, io, iter::zip, rc::Rc};

fn move_by(i: usize, δ: isize, l: usize) -> usize {
    match i.saturating_add_signed(δ) >= l {
        true => l - 1,
        false => i.saturating_add_signed(δ),
    }
}

#[derive(Debug)]
enum Popup {
    Book,
    Filter,
}

struct App<'b> {
    bookcase: &'b mut Bookcase,
    popup: Option<Popup>,
    visible_books: Vec<usize>,
    state: TableState,
}

impl<'b> App<'b> {
    fn new(bookcase: &'b mut Bookcase) -> App {
        let visible_books = bookcase.books.keys().cloned().collect();
        App {
            bookcase,
            popup: None,
            visible_books,
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
                Popup::Book => {
                    if let Some(i) = app.state.selected() {
                        if let Some(b) = app.bookcase.get_book(app.visible_books[i]) {
                            let returned_book = run_popup_book(terminal, b)?;
                            if let Some(book) = returned_book {
                                app.bookcase.books.insert(app.visible_books[i], book);
                            }
                        };
                    };
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
                        Enter => app.popup = Some(Popup::Book),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn draw(rect: &mut Frame, app: &mut App) {
    let size = rect.size();
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(2)]).split(size);

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
            Constraint::Length(3),
            Constraint::Min(35),
            Constraint::Length(35),
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

enum SelectionChange {
    Select,
    Deselect,
    Toggle,
}

struct SelectableList<T> {
    values: Vec<T>,
    selected: Vec<bool>,
    cursor_position: usize,
    state: ListState,
    len: usize,
}

impl<T> SelectableList<T>
where
    T: Clone + Display,
{
    fn new(values: &[T]) -> Self {
        let len = values.len();
        let selected = vec![false; len];
        let values = values.into_iter().cloned().collect();
        SelectableList {
            values,
            selected,
            cursor_position: 0,
            state: ListState::default(),
            len,
        }
    }
    fn move_by(&mut self, δ: isize) {
        self.cursor_position = move_by(self.cursor_position, δ, self.len);
        self.state.select(Some(self.cursor_position));
    }
    fn change_selection(&mut self, switch: SelectionChange) {
        if let Some(b) = self.selected.get_mut(self.cursor_position) {
            match switch {
                SelectionChange::Select => *b = true,
                SelectionChange::Deselect => *b = false,
                SelectionChange::Toggle => *b = !*b,
            };
        }
    }
    fn activate(&mut self) {
        self.state.select(Some(self.cursor_position))
    }
    fn deactivate(&mut self) {
        self.state.select(None)
    }
    fn as_stateful_list(&mut self) -> (List, &mut ListState) {
        let author_list = List::new(self.values.iter().enumerate().map(|(i, a)| {
            let selected = selected_symbol(self.selected[i]);
            format!("[{selected}] {a}")
        }));
        (author_list, &mut self.state)
    }
}

struct FilterPopupApp {
    authors: SelectableList<Rc<str>>,
    read: SelectableList<Read>,
    current_field: FilterPopupField,
}

impl FilterPopupApp {
    fn new(books: &Bookcase) -> Self {
        let read = SelectableList::new(&Read::all());
        let author_list: Vec<_> = books.get_authors().iter().map(|a| Rc::from(*a)).collect();
        let mut authors = SelectableList::new(&author_list);
        authors.activate();
        FilterPopupApp {
            authors,
            read,
            current_field: FilterPopupField::Author,
        }
    }
    fn move_by(&mut self, δ: isize) {
        match self.current_field {
            FilterPopupField::Author => self.authors.move_by(δ),
            FilterPopupField::Read => self.read.move_by(δ),
        }
    }
    fn toggle(&mut self) {
        match self.current_field {
            FilterPopupField::Author => self.authors.change_selection(SelectionChange::Toggle),
            FilterPopupField::Read => self.read.change_selection(SelectionChange::Toggle),
        }
    }
    fn deselect(&mut self) {
        match self.current_field {
            FilterPopupField::Author => self.authors.change_selection(SelectionChange::Deselect),
            FilterPopupField::Read => self.read.change_selection(SelectionChange::Deselect),
        }
    }
    fn switch_fields(&mut self, new_field: FilterPopupField) {
        match self.current_field {
            FilterPopupField::Author => self.authors.deactivate(),
            FilterPopupField::Read => self.read.deactivate(),
        }
        self.current_field = new_field;
        match self.current_field {
            FilterPopupField::Author => self.authors.activate(),
            FilterPopupField::Read => self.read.activate(),
        }
    }
    fn tab(&mut self) {
        self.switch_fields(self.current_field.next())
    }
    fn to_filter(self) -> Filter {
        Filter {
            author_match: zip(self.authors.values, self.authors.selected)
                .filter_map(|(a, b)| b.then_some(a))
                .collect(),
            read: zip(self.read.values, self.read.selected)
                .filter_map(|(r, b)| b.then_some(r))
                .collect(),
        }
    }
}

fn run_popup_filter<'f, B: Backend>(
    terminal: &mut Terminal<B>,
    books: &'f Bookcase,
) -> Result<Option<Filter>, io::Error> {
    let mut app_popup = FilterPopupApp::new(books);
    loop {
        terminal.draw(|rect| draw_popup_filter(rect, &mut app_popup))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Enter => return Ok(Some(app_popup.to_filter())),
                        Char('k') | Up => app_popup.move_by(-1),
                        Char('j') | Down => app_popup.move_by(1),
                        Esc => return Ok(None),
                        Backspace | Delete => app_popup.deselect(),
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

    let author_block = Block::bordered().title("Author");
    let (author_list, author_state) = app.authors.as_stateful_list();
    let author_list = author_list
        .block(author_block)
        .highlight_style(highlight_style);
    f.render_stateful_widget(author_list, popup_filter_layout[0], author_state);

    let read_block = Block::bordered().title("Read");
    let (read_list, read_state) = app.read.as_stateful_list();
    let read_list = read_list.block(read_block).highlight_style(highlight_style);
    f.render_stateful_widget(read_list, popup_filter_layout[1], read_state);
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

#[derive(PartialEq)]
enum BookPopupField {
    Title,
    Author,
    Read,
}

struct BookPopupApp {
    book: Book,
    current_field: BookPopupField,
}

impl BookPopupApp {
    fn new(book: &Book) -> Self {
        BookPopupApp {
            book: book.clone(),
            current_field: BookPopupField::Title,
        }
    }
    fn tab(&mut self) {
        self.current_field = match self.current_field {
            BookPopupField::Title => BookPopupField::Author,
            BookPopupField::Author => BookPopupField::Read,
            BookPopupField::Read => BookPopupField::Title,
        }
    }
    fn backspace(&mut self) {
        match self.current_field {
            BookPopupField::Author => {
                self.book.author.pop();
            }
            BookPopupField::Title => {
                self.book.title.pop();
            }
            _ => {}
        }
    }
    fn input(&mut self, value: char) {
        match self.current_field {
            BookPopupField::Author => self.book.author.push(value),
            BookPopupField::Title => self.book.title.push(value),
            _ => {}
        }
    }
}

fn run_popup_book<'b, B: Backend>(
    terminal: &mut Terminal<B>,
    book: &'b Book,
) -> Result<Option<Book>, io::Error> {
    let mut app_popup = BookPopupApp::new(book);
    loop {
        terminal.draw(|rect| draw_popup_book(rect, &mut app_popup))?;
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    use KeyCode::*;
                    match key.code {
                        Enter => return Ok(Some(app_popup.book)),
                        Esc => return Ok(None),
                        Tab => app_popup.tab(),
                        Backspace => app_popup.backspace(),
                        Char(value) => app_popup.input(value),
                        _ => {}
                    }
                }
            }
        }
    }
}

fn draw_popup_book(f: &mut Frame, app: &mut BookPopupApp) {
    let area = popup_rect(80, 80, f.size());

    let popup_block = ratatui::widgets::Block::default().title("Book");
    f.render_widget(popup_block, area);

    let popup_filter_layout_vertical = Layout::vertical([
        Constraint::Min(3),
        Constraint::Min(3),
        Constraint::Min(3),
        Constraint::Fill(1),
    ]);
    let popup_filter_layout = popup_filter_layout_vertical.split(area);

    let block_selected_style = Style::default().fg(Color::Yellow);

    let title_block = block_border_style_if(
        Block::bordered().title("Title"),
        app.current_field == BookPopupField::Title,
        block_selected_style,
    );
    let author_block = block_border_style_if(
        Block::bordered().title("Author"),
        app.current_field == BookPopupField::Author,
        block_selected_style,
    );
    let read_block = block_border_style_if(
        Block::bordered().title("Read"),
        app.current_field == BookPopupField::Read,
        block_selected_style,
    );

    let title = Paragraph::new(app.book.title.as_str()).block(title_block);
    let read = Paragraph::new(app.book.read_state().to_string()).block(read_block);
    let author = Paragraph::new(app.book.author.as_str()).block(author_block);

    f.render_widget(title, popup_filter_layout[0]);
    f.render_widget(author, popup_filter_layout[1]);
    f.render_widget(read, popup_filter_layout[2]);
}

fn block_border_style_if(block: Block, cond: bool, style: Style) -> Block {
    match cond {
        true => block.border_style(style),
        false => block,
    }
}
