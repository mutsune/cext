use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// cext - Chrome unpacked/private extension manager backed by git
#[derive(Parser)]
#[command(name = "cext", version, about, long_about = None)]
pub struct Cli {
    /// Override the extensions storage directory
    /// (default: $HOME/Library/Application Support/Google/private extensions/)
    #[arg(long, global = true, value_name = "DIR")]
    pub dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clone a Chrome extension from a git remote URL and save it
    Add {
        /// git remote URL (https:// or git@... etc.)
        url: String,

        /// Folder name to save under (default: whatever `git clone` names it,
        /// same as running `git clone <url>` without a destination argument)
        #[arg(long)]
        name: Option<String>,
    },

    /// List saved extensions as a plain URL text list
    List {
        /// Write the list to a file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Import a URL list file and save every extension listed in it
    Import {
        /// Path to a text file with one git URL per line
        /// (blank lines and lines starting with '#' are ignored)
        file: PathBuf,
    },

    /// Remove a saved extension by name
    Remove {
        /// Name of the saved extension (its folder name under the storage dir)
        name: String,

        /// Skip the confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}
