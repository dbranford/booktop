use clap::{Args, Parser, Subcommand};
use std::fs::File;
use std::path::PathBuf;
mod books;
mod util;

#[derive(Debug, Parser)]
#[clap(about = "A basic tracker for books", long_about = None)]
struct Cli {
    #[clap(long, short)]
    /// File containing existing bookcase
    file: Option<PathBuf>,

    #[clap(long, takes_value = false)]
    /// Do not attempt to open a (default) file
    no_file: bool,

    #[clap(long, takes_value = false)]
    /// Run commands without updating the file
    dry_run: bool,

    #[clap(short, long, takes_value = false)]
    /// Follow command with list
    list: bool,

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
    /// Pick a book at random
    Pick {},
    /// Start reading a book
    Start(books::BookQuery),
    /// Finish reading a book
    Finish(books::BookQuery),
    /// Pause reading a book
    Stop(books::BookQuery),
    /// Return a book to unread
    Reset(books::BookQuery),
    /// Use a utility function
    Util(Util),
}

#[derive(Debug, Args)]
struct Util {
    #[clap(subcommand)]
    command: UtilCommands,

    #[clap(short, long, takes_value = false)]
    write: bool,
}

#[derive(Debug, Subcommand)]
enum UtilCommands {
    /// Set books to be a non-empty example bookcase
    ExampleBookcase,
    /// Re-index bookcase, reassigning no longer active keys
    Renumber,
}

fn main() {
    let args = Cli::parse();

    let mut write = !args.dry_run;

    let default_file_path = PathBuf::from("bookcase.booktop.json");

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
        Commands::List {} => {
            books.list();
        }
        Commands::Remove { id } => {
            books.remove_book(id);
        }
        Commands::Pick {} => {
            let picked = books.pick_book();
            println!("{} | {}", picked.0, picked.1)
        }
        // Book operations
        Commands::Finish(bookquery) => {
            if let Some(book) = books.get_mut_book(bookquery) {
                book.finish();
            }
        }
        Commands::Start(bookquery) => {
            if let Some(book) = books.get_mut_book(bookquery) {
                book.start();
            }
        }
        Commands::Reset(bookquery) => {
            if let Some(book) = books.get_mut_book(bookquery) {
                book.reset()
            }
        }
        Commands::Stop(bookquery) => {
            if let Some(book) = books.get_mut_book(bookquery) {
                book.stop()
            }
        }
        Commands::Util(util) => {
            write = util.write;
            match util.command {
                UtilCommands::ExampleBookcase {} => books = util::example_bookcase(),
                UtilCommands::Renumber {} => {
                    books.util_renumber();
                    books.list()
                }
            }
        }
    }

    if args.list {
        books.list();
    }

    if write {
        match &file_path {
            Some(path) => books.close(path),
            None => (),
        };
    }
}
