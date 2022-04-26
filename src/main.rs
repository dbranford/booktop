use clap::{Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
mod books;

#[derive(Debug, Parser)]
#[clap(about = "A basic tracker for books", long_about = None)]
struct Cli {
    #[clap(long, short)]
    /// File containing existing bookcase
    file: Option<PathBuf>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a book
    Add { title: String, author: String },
    /// List all books
    List {},
    /// Initialise bookcase file
    Init { path: PathBuf },
    /// Remove book
    Remove { id: usize },
    /// Start reading a book
    Start { id: usize },
    /// Finish reading a book
    Finish { id: usize },
    /// Pause reading a book
    Stop { id: usize },
    /// Return a book to unread
    Reset { id: usize },
}

fn main() {
    let args = Cli::parse();

    let default_file_path = PathBuf::from("bookcase.booktop.json");

    let mut file_path = match args.file {
        Some(path) => Some(path),
        None => match default_file_path.is_file() {
            true => Some(default_file_path),
            false => None,
        },
    };

    let mut books = match &file_path {
        Some(path) => books::Bookcase::open(path),
        None => books::Bookcase::new(),
    };

    match args.command {
        // Bookcase operations
        Commands::Add { title, author } => {
            books.add_book(title, author);
        }
        Commands::Init { path } => {
            File::create(&path).expect("Could not create file");
            file_path = Some(path);
        }
        Commands::List {} => {
            books.list();
        }
        Commands::Remove { id } => {
            books.remove_book(id);
        }
        // Book operations
        Commands::Finish { id } => {
            if let Some(book) = books.books.get_mut(&id) {
                book.finish();
            }
        }
        Commands::Start { id } => {
            if let Some(book) = books.books.get_mut(&id) {
                book.start();
            }
        }
        Commands::Reset { id } => {
            if let Some(book) = books.books.get_mut(&id) {
                book.reset()
            }
        }
        Commands::Stop { id } => {
            if let Some(book) = books.books.get_mut(&id) {
                book.stop()
            }
        }
    }

    match &file_path {
        Some(path) => books.close(path),
        None => (),
    };
}
