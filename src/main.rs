use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
mod book;
mod books;
mod filter;
mod tui;
mod util;

#[derive(Debug, Parser)]
#[command(about = "A basic tracker for books", long_about = None)]
struct Cli {
    #[arg(long, short)]
    /// File containing existing bookcase
    file: Option<PathBuf>,

    #[arg(long, num_args = 0)]
    /// Do not attempt to open a (default) file
    no_file: bool,

    #[arg(long, num_args = 0)]
    /// Run commands without updating the file
    dry_run: bool,

    #[arg(short, long, num_args = 0)]
    /// Follow command with list
    list: bool,

    #[command(subcommand)]
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
    /// Pick a book at random
    Pick {},
    /// Start reading a book
    Start { id: usize },
    /// Finish reading a book
    Finish { id: usize },
    /// Pause reading a book
    Stop { id: usize },
    /// Return a book to unread
    Reset { id: usize },
    /// Use a utility function
    Util(Util),
    /// Start UI
    Tui { file: Option<PathBuf> },
}

#[derive(Debug, Args)]
struct Util {
    #[command(subcommand)]
    command: UtilCommands,

    #[arg(short, long, num_args = 0)]
    write: bool,
}

#[derive(Debug, Subcommand)]
enum UtilCommands {
    /// Set books to be a non-empty example bookcase
    ExampleBookcase,
    /// Re-index bookcase, reassigning no longer active keys
    Renumber,
}

fn list(books: &books::Bookcase) {
    println!("Bookcase: {}", books.name);
    println!("========================================");
    for (id, bk) in books.get_books() {
        println!("{}: {}", id, bk);
    }
}

fn main() {
    let args = Cli::parse();

    let mut write = !args.dry_run;

    let default_file_path = PathBuf::from("bookcase.booktop.yaml");

    let mut file_path = match args.file {
        Some(path) => Some(path),
        None => match default_file_path.is_file() {
            true => Some(default_file_path),
            false => None,
        },
    };

    let mut books = match (&file_path, args.no_file) {
        (Some(path), false) => books::Bookcase::open(path),
        (_, _) => books::Bookcase::new(),
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
        Commands::List {} => list(&books),
        Commands::Remove { id } => {
            books.remove_book(id);
        }
        Commands::Pick {} => {
            let picked = books.pick_book();
            println!("{} | {}", picked.0, picked.1)
        }
        // Book operations
        Commands::Finish { id } => {
            if let Some(book) = books.get_mut_book(id) {
                book.finish();
            }
        }
        Commands::Start { id } => {
            if let Some(book) = books.get_mut_book(id) {
                book.start();
            }
        }
        Commands::Reset { id } => {
            if let Some(book) = books.get_mut_book(id) {
                book.reset()
            }
        }
        Commands::Stop { id } => {
            if let Some(book) = books.get_mut_book(id) {
                book.stop()
            }
        }
        Commands::Util(util) => {
            write = util.write;
            match util.command {
                UtilCommands::ExampleBookcase {} => books = util::example_bookcase(),
                UtilCommands::Renumber {} => {
                    books.util_renumber();
                    list(&books)
                }
            }
        }
        Commands::Tui { file } => {
            if let Some(file) = file {
                books = books::Bookcase::open(file)
            };
            tui::start_tui(&mut books).ok();
        }
    }

    if args.list {
        list(&books)
    }

    if write {
        match &file_path {
            Some(path) => books.close(path),
            None => (),
        };
    }
}
