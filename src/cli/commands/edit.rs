use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;

use crate::api::{load_image_base64, GeminiClient};
use crate::config::Config;
use crate::core::GenerateParams;
use crate::core::Job;
use crate::db::Database;

#[derive(Args)]
pub struct EditArgs {
    /// Path to the image to edit
    #[arg(required = true)]
    pub image: PathBuf,

    /// The edit instruction (e.g., "make the sky blue", "add a hat")
    #[arg(required = true)]
    pub prompt: String,

    /// Aspect ratio for the output
    #[arg(short, long, alias = "ar")]
    pub aspect_ratio: Option<String>,

    /// Image size (1K, 2K, 4K)
    #[arg(short, long)]
    pub size: Option<String>,

    /// Model to use
    #[arg(short, long)]
    pub model: Option<String>,

    /// Output directory for edited images
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Don't download images automatically
    #[arg(long)]
    pub no_download: bool,

    /// Output format (text, json, quiet)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub async fn run(args: EditArgs, config: &Config, db: &Database) -> Result<()> {
    // Load the source image
    let image_path = args.image.canonicalize()
        .context("Image file not found")?;

    let (base64_data, mime_type) = load_image_base64(&image_path).await
        .context("Failed to load image file")?;

    // Build parameters with reference image
    let params = GenerateParams::new(&args.prompt)
        .with_aspect_ratio(args.aspect_ratio.as_deref().unwrap_or(&config.defaults.aspect_ratio))
        .with_size(args.size.as_deref().unwrap_or(&config.defaults.size))
        .with_model(args.model.as_deref().unwrap_or(&config.api.model))
        .with_reference_image(base64_data, mime_type);

    // Create job
    let mut job = Job::new_edit(params, image_path.to_string_lossy().to_string());

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
        pb.set_message(format!("Editing image: {}...", job.prompt_preview(40)));
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // Set job as running
    job.set_running(0);
    db.update_job(&job)?;

    // Generate edited image
    match client.generate(&job.params).await {
        Ok(response) => {
            if let Err(e) = client.process_response(&mut job, response) {
                job.set_failed(e.to_string());
                db.update_job(&job)?;

                if let Some(pb) = pb {
                    pb.finish_with_message(format!("{} Edit failed", "✗".red()));
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
                pb.finish_with_message(format!("{} Edit failed", "✗".red()));
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
                "{} Edited image saved",
                "✓".green()
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
                println!("{}: {}", "Source".cyan().bold(), image_path.display());
                println!("{}: {}", "Edit".cyan().bold(), job.params.prompt);
                println!("{}: {}", "Model".cyan().bold(), job.model);
                println!("{}: {}", "Status".cyan().bold(), "completed".green());
                println!();
                println!("{}:", "Edited Image".cyan().bold());
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
                "{} Edit complete (not downloaded)",
                "✓".green()
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
