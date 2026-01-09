use anyhow::Result;
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

use crate::api::GeminiClient;
use crate::config::Config;
use crate::core::GenerateParams;
use crate::core::Job;
use crate::db::Database;

#[derive(Args)]
pub struct GenerateArgs {
    /// The prompt describing the image to generate
    #[arg(required = true)]
    pub prompt: String,

    /// Aspect ratio (1:1, 2:3, 3:2, 3:4, 4:3, 4:5, 5:4, 9:16, 16:9, 21:9)
    #[arg(short, long, alias = "ar")]
    pub aspect_ratio: Option<String>,

    /// Image size (1K, 2K, 4K - 4K only for Gemini 3 Pro)
    #[arg(short, long)]
    pub size: Option<String>,

    /// Model to use
    #[arg(short, long)]
    pub model: Option<String>,

    /// Output directory for downloaded images
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Don't download images automatically
    #[arg(long)]
    pub no_download: bool,

    /// Output format (text, json, quiet)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub async fn run(args: GenerateArgs, config: &Config, db: &Database) -> Result<()> {
    // Build parameters
    let params = GenerateParams::new(&args.prompt)
        .with_aspect_ratio(args.aspect_ratio.as_deref().unwrap_or(&config.defaults.aspect_ratio))
        .with_size(args.size.as_deref().unwrap_or(&config.defaults.size))
        .with_model(args.model.as_deref().unwrap_or(&config.api.model));

    // Create job
    let mut job = Job::new_generate(params);

    // Save to database
    db.insert_job(&job)?;

    // Create API client
    let client = GeminiClient::from_config(config)?;

    // Show progress
    let pb = if args.format == "text" {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.yellow} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Generating image: {}...", job.prompt_preview(40)));
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // Set job as running
    job.set_running(0);
    db.update_job(&job)?;

    // Generate
    match client.generate(&job.params).await {
        Ok(response) => {
            if let Err(e) = client.process_response(&mut job, response) {
                job.set_failed(e.to_string());
                db.update_job(&job)?;

                if let Some(pb) = pb {
                    pb.finish_with_message(format!("{} Generation failed", "✗".red()));
                }

                if args.format == "json" {
                    println!("{}", serde_json::to_string_pretty(&job)?);
                } else if args.format != "quiet" {
                    eprintln!("{}: {}", "Error".red().bold(), e);
                }
                return Err(e);
            }
        }
        Err(e) => {
            job.set_failed(e.to_string());
            db.update_job(&job)?;

            if let Some(pb) = pb {
                pb.finish_with_message(format!("{} Generation failed", "✗".red()));
            }

            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else if args.format != "quiet" {
                eprintln!("{}: {}", "Error".red().bold(), e);
            }
            return Err(e);
        }
    }

    // Download images
    let output_dir = args
        .output
        .unwrap_or_else(|| PathBuf::from(&config.output.directory));

    if !args.no_download && config.output.auto_download {
        let paths = client.download_images(&mut job, &output_dir).await?;

        if let Some(pb) = &pb {
            pb.finish_with_message(format!(
                "{} Generated {} image(s)",
                "✓".green(),
                paths.len()
            ));
        }

        // Display based on format
        match args.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&job)?);
            }
            "quiet" => {
                for path in &paths {
                    println!("{}", path);
                }
            }
            _ => {
                println!();
                println!("{}: {}", "Job ID".cyan().bold(), job.id);
                println!("{}: {}", "Prompt".cyan().bold(), job.params.prompt);
                println!("{}: {}", "Model".cyan().bold(), job.model);
                println!("{}: {}", "Aspect Ratio".cyan().bold(), job.params.aspect_ratio);
                println!("{}: {}", "Status".cyan().bold(), "completed".green());
                println!();
                println!("{}:", "Generated Images".cyan().bold());
                for path in &paths {
                    println!("  {}", path);
                }

                // Try to display image in terminal
                if config.output.display == crate::config::DisplayMode::Terminal {
                    if let Some(first_path) = paths.first() {
                        println!();
                        display_image_terminal(first_path);
                    }
                }
            }
        }
    } else {
        if let Some(pb) = &pb {
            pb.finish_with_message(format!(
                "{} Generated {} image(s) (not downloaded)",
                "✓".green(),
                job.images.len()
            ));
        }

        if args.format == "json" {
            println!("{}", serde_json::to_string_pretty(&job)?);
        }
    }

    // Update database
    db.update_job(&job)?;

    Ok(())
}

/// Display an image in the terminal using viuer
fn display_image_terminal(path: &str) {
    let conf = viuer::Config {
        width: Some(80),
        height: Some(30),
        absolute_offset: false,
        ..Default::default()
    };

    if let Err(e) = viuer::print_from_file(path, &conf) {
        tracing::debug!("Failed to display image in terminal: {}", e);
    }
}
