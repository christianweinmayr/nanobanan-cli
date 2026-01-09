use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::config::Config;

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: Option<ConfigCommand>,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Show all configuration values
    Show,

    /// Get a specific configuration value
    Get {
        /// Config key (e.g., api.key, defaults.aspect_ratio)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Config key (e.g., api.key, defaults.aspect_ratio)
        key: String,
        /// Value to set
        value: String,
    },

    /// Show the config file path
    Path,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

pub fn run(args: ConfigArgs, config: &mut Config) -> Result<()> {
    match args.command {
        Some(ConfigCommand::Show) | None => show_config(config),
        Some(ConfigCommand::Get { key }) => get_config(&key, config),
        Some(ConfigCommand::Set { key, value }) => set_config(&key, &value, config),
        Some(ConfigCommand::Path) => show_path(config),
        Some(ConfigCommand::Reset { force }) => reset_config(force, config),
    }
}

fn show_config(config: &Config) -> Result<()> {
    println!("{}", "Configuration".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();

    println!("[{}]", "api".yellow());
    println!("  {} = {}", "key".bold(), config.get("api.key").unwrap_or_else(|| "(not set)".dimmed().to_string()));
    println!("  {} = {}", "model".bold(), config.api.model);
    println!("  {} = {}", "base_url".bold(), config.api.base_url);
    println!();

    println!("[{}]", "defaults".yellow());
    println!("  {} = {}", "aspect_ratio".bold(), config.defaults.aspect_ratio);
    println!("  {} = {}", "size".bold(), config.defaults.size);
    println!();

    println!("[{}]", "output".yellow());
    println!("  {} = {}", "directory".bold(), config.output.directory);
    println!("  {} = {}", "auto_download".bold(), config.output.auto_download);
    println!("  {} = {}", "display".bold(), config.output.display.as_str());
    println!();

    println!("[{}]", "tui".yellow());
    println!("  {} = {}", "show_images".bold(), config.tui.show_images);
    println!("  {} = {}", "theme".bold(), config.tui.theme);
    println!();

    println!("{}", format!("Config file: {}", config.config_path.display()).dimmed());

    Ok(())
}

fn get_config(key: &str, config: &Config) -> Result<()> {
    match config.get(key) {
        Some(value) => println!("{}", value),
        None => {
            eprintln!("{}: Unknown config key '{}'", "Error".red().bold(), key);
            eprintln!();
            eprintln!("Available keys:");
            for k in Config::keys() {
                eprintln!("  {}", k);
            }
        }
    }
    Ok(())
}

fn set_config(key: &str, value: &str, config: &mut Config) -> Result<()> {
    config.set(key, value)?;
    config.save()?;

    println!("{} Set {} = {}", "✓".green(), key.cyan(), value);
    Ok(())
}

fn show_path(config: &Config) -> Result<()> {
    println!("{}", config.config_path.display());
    Ok(())
}

fn reset_config(force: bool, config: &mut Config) -> Result<()> {
    if !force {
        eprintln!(
            "{}: This will reset all configuration to defaults. Use --force to confirm.",
            "Warning".yellow().bold()
        );
        return Ok(());
    }

    // Preserve the path
    let path = config.config_path.clone();

    // Reset to defaults
    *config = Config::default();
    config.config_path = path;

    // Check for env var
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        config.api.key = Some(key);
    }

    config.save()?;

    println!("{} Configuration reset to defaults", "✓".green());
    Ok(())
}
