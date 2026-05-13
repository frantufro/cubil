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

    /// Move a task from backlog/ to doing/
    Start { slug: String },

    /// Move a task from doing/ to done/
    Finish { slug: String },

    /// Delete a task
    Rm { slug: String },

    /// Upgrade the cubil binary to the latest GitHub release
    Update,

    /// Manage roadmaps (ordered, milestone-divided task sequences)
    Roadmap {
        #[command(subcommand)]
        command: RoadmapCommands,
    },
}

#[derive(Subcommand)]
enum RoadmapCommands {
    /// Create a new roadmap under .cubil/roadmaps/
    ///
    /// Prints the resulting slug to stdout on success. Body may be supplied
    /// inline via -m, from a file via -F <path>, or from stdin via -F -.
    New {
        /// Roadmap title (slug derived automatically)
        title: String,
        /// Inline body
        #[arg(short = 'm', long, conflicts_with = "file")]
        message: Option<String>,
        /// Read body from file; use `-` for stdin
        #[arg(short = 'F', long, value_name = "PATH")]
        file: Option<PathBuf>,
    },

    /// List all roadmaps
    List {
        /// Emit JSON instead of a human table
        #[arg(long)]
        json: bool,
    },

    /// Render a roadmap with task statuses resolved (and rewrite the file)
    Show { slug: String },

    /// Append a task reference to a roadmap
    Add {
        /// Roadmap slug
        roadmap: String,
        /// Task slug to add (must exist in some status folder)
        task: String,
        /// Append to the named milestone instead of the last one
        #[arg(long, value_name = "NAME")]
        milestone: Option<String>,
    },

    /// Print the slug of the next not-done task in the roadmap
    Next { slug: String },

    /// Delete a roadmap
    Rm { slug: String },
}

fn main() {
    let cli = Cli::parse();

    // Stale-version check runs before non-update commands. Best-effort: any
    // failure (timeout, network down, malformed cache) is silently dropped.
    if !matches!(cli.command, Commands::Update) {
        if let Some(warning) = core::updater::stale_warning() {
            eprintln!("{warning}");
        }
    }

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
        Commands::Start { slug } => commands::start::run(slug),
        Commands::Finish { slug } => commands::finish::run(slug),
        Commands::Rm { slug } => commands::rm::run(slug),
        Commands::Update => commands::update::run(),
        Commands::Roadmap { command } => match command {
            RoadmapCommands::New {
                title,
                message,
                file,
            } => commands::roadmap::new::run(title, message, file),
            RoadmapCommands::List { json } => commands::roadmap::list::run(json),
            RoadmapCommands::Show { slug } => commands::roadmap::show::run(slug),
            RoadmapCommands::Add {
                roadmap,
                task,
                milestone,
            } => commands::roadmap::add::run(roadmap, task, milestone),
            RoadmapCommands::Next { slug } => commands::roadmap::next::run(slug),
            RoadmapCommands::Rm { slug } => commands::roadmap::rm::run(slug),
        },
    };
    if let Err(e) = result {
        eprintln!("{e}");
        process::exit(1);
    }
}
