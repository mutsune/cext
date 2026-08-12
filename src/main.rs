mod cli;
mod ops;

use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let store_dir = cli.dir.clone().unwrap_or_else(ops::default_store_dir);

    match cli.command {
        Commands::Add { url, name } => ops::add(&store_dir, &url, name.as_deref()),
        Commands::List { output } => ops::list(&store_dir, output.as_deref()),
        Commands::Import { file } => ops::import(&store_dir, &file),
        Commands::Remove { name, yes } => ops::remove(&store_dir, &name, yes),
    }
}
