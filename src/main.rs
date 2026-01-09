use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod api;
mod cli;
mod config;
mod core;
mod db;
mod http_client;
mod tui;

use cli::{Cli, Commands};
use config::Config;
use db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let cli = Cli::parse();

    // Load or create config
    let mut config = Config::load_or_create()?;

    // Initialize database
    let db = Database::open()?;

    match cli.command {
        Some(Commands::Generate(args)) => {
            cli::commands::generate::run(args, &config, &db).await?;
        }
        Some(Commands::Edit(args)) => {
            cli::commands::edit::run(args, &config, &db).await?;
        }
        Some(Commands::Jobs(args)) => {
            cli::commands::jobs::run(args, &db)?;
        }
        Some(Commands::Config(args)) => {
            cli::commands::config::run(args, &mut config)?;
        }
        None => {
            // Launch TUI
            tui::run(&mut config, &db).await?;
        }
    }

    Ok(())
}
