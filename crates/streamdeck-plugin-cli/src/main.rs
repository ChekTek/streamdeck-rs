mod commands;
mod project;
mod stream_deck;
mod template;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "streamdeck-plugin",
    version,
    about = "Create and manage native Stream Deck plugins written in Rust"
)]
struct Cli {
    /// Display list of installed plugins
    #[arg(short = 'l', long = "list")]
    list: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Stream Deck plugin creation wizard
    Create,
    /// Links the plugin to Stream Deck
    Link {
        /// Path of the plugin to link
        path: Option<PathBuf>,
    },
    /// Unlinks the plugin from Stream Deck
    Unlink {
        uuid: String,
        /// Enable deletion of non-linked plugins
        #[arg(short, long)]
        delete: bool,
    },
    /// Display list of installed plugins
    List {
        /// Show all plugins, not only linked ones
        #[arg(short, long)]
        all: bool,
    },
    /// Starts the plugin in Stream Deck; if already running, it is stopped first
    #[command(visible_alias = "r")]
    Restart { uuid: String },
    /// Stops the plugin in Stream Deck
    #[command(visible_alias = "s")]
    Stop { uuid: String },
    /// Enables developer mode
    Dev {
        /// Disables developer mode
        #[arg(short, long)]
        disable: bool,
    },
    /// Validates the Stream Deck plugin
    Validate {
        /// Path of the plugin to validate
        path: Option<PathBuf>,
    },
    /// Creates a .streamDeckPlugin file from the plugin
    #[command(visible_alias = "bundle")]
    Pack {
        /// Path of the plugin to pack
        path: Option<PathBuf>,
        /// Generates a report without creating a package
        #[arg(long)]
        dry_run: bool,
        /// Overwrite an existing package
        #[arg(short, long)]
        force: bool,
        /// Output directory for the package
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Plugin version written to the manifest before packing
        #[arg(long)]
        version: Option<String>,
        /// Bypass validation errors (not recommended)
        #[arg(long)]
        ignore_validation: bool,
    },
    /// Build the plugin and copy the binary into the .sdPlugin folder
    Build {
        /// Path of the plugin project or .sdPlugin directory
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Create) => commands::create::run(),
        Some(Commands::Link { path }) => commands::link::run(path),
        Some(Commands::Unlink { uuid, delete }) => commands::unlink::run(&uuid, delete),
        Some(Commands::List { all }) => commands::list::run(all),
        Some(Commands::Restart { uuid }) => commands::restart::run(&uuid, false),
        Some(Commands::Stop { uuid }) => commands::stop::run(&uuid),
        Some(Commands::Dev { disable }) => commands::dev::run(disable),
        Some(Commands::Validate { path }) => commands::validate::run(path),
        Some(Commands::Pack {
            path,
            dry_run,
            force,
            output,
            version,
            ignore_validation,
        }) => commands::pack::run(path, dry_run, force, output, version, ignore_validation),
        Some(Commands::Build { path }) => commands::build::run(path),
        None if cli.list => commands::list::run(false),
        None => {
            use clap::CommandFactory;
            let _ = Cli::command().print_help();
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
