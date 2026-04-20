use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

mod commands;
// Most of `core`'s API is consumed by commands that land in follow-up PRs
// (`new`, `show`, `edit`, `mv`, `rm`). Allow dead code at the module root so
// the public API can ship stably before its first callers exist.
#[allow(dead_code)]
mod core;

#[derive(Parser)]
#[command(
    name = "cubil",
    version,
    about = "Markdown-based task management — companion to Skulk",
    long_about = "Cubil manages tasks as Markdown files in a .cubil/ directory.\n\nEach subdirectory of .cubil/ is a status (backlog, doing, done, ...). Tasks\nare plain Markdown files with optional YAML frontmatter. Cubil is agent-first:\n`new` takes a title plus a body via -m, -F, or stdin, and prints the slug to\nstdout. Moves between statuses are explicit via `mv`."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .cubil/ with default status folders (backlog, doing, done)
    Init,

    /// Create a new task in backlog/
    ///
    /// Prints the resulting slug to stdout on success. Body may be supplied
    /// inline via -m, from a file via -F <path>, or from stdin via -F -.
    New {
        /// Task title (human-readable; slug derived automatically)
        title: String,
        /// Inline body
        #[arg(short = 'm', long, conflicts_with = "file")]
        message: Option<String>,
        /// Read body from file; use `-` for stdin
        #[arg(short = 'F', long, value_name = "PATH")]
        file: Option<PathBuf>,
    },

    /// List tasks (active statuses by default)
    List {
        /// Include tasks in `done/`
        #[arg(long)]
        all: bool,
        /// Show only tasks in the given status folder
        #[arg(long, value_name = "STATUS")]
        status: Option<String>,
        /// Emit JSON instead of a human table
        #[arg(long)]
        json: bool,
    },

    /// Print a task's full markdown to stdout
    Show { slug: String },

    /// Open a task in $EDITOR (falls back to vi)
    Edit { slug: String },

    /// Move a task to a different status folder
    Mv { slug: String, status: String },

    /// Delete a task
    Rm { slug: String },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New {
            title,
            message,
            file,
        } => commands::new::run(title, message, file),
        Commands::List { all, status, json } => commands::list::run(all, status, json),
        Commands::Show { slug } => commands::show::run(slug),
        Commands::Edit { slug } => commands::edit::run(slug),
        Commands::Mv { slug, status } => commands::mv::run(slug, status),
        Commands::Rm { slug } => commands::rm::run(slug),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        process::exit(1);
    }
}
