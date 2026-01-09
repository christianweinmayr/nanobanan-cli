use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;

use crate::db::Database;

#[derive(Args)]
pub struct JobsArgs {
    #[command(subcommand)]
    pub command: Option<JobsCommand>,

    /// Maximum number of jobs to show
    #[arg(short, long, default_value = "20")]
    pub limit: u32,

    /// Filter by status (queued, running, completed, failed, cancelled)
    #[arg(short, long)]
    pub status: Option<String>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

#[derive(Subcommand)]
pub enum JobsCommand {
    /// Show detailed information about a specific job
    Show {
        /// Job ID
        job_id: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Delete a job from history
    Delete {
        /// Job ID
        job_id: String,
    },

    /// Clear all jobs from history
    Clear {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

pub fn run(args: JobsArgs, db: &Database) -> Result<()> {
    match args.command {
        Some(JobsCommand::Show { job_id, format }) => show_job(&job_id, &format, db),
        Some(JobsCommand::Delete { job_id }) => delete_job(&job_id, db),
        Some(JobsCommand::Clear { force }) => clear_jobs(force, db),
        None => list_jobs(args.limit, args.status.as_deref(), &args.format, db),
    }
}

fn list_jobs(limit: u32, status: Option<&str>, format: &str, db: &Database) -> Result<()> {
    let jobs = db.list_jobs(limit, status)?;

    if jobs.is_empty() {
        if format == "json" {
            println!("[]");
        } else {
            println!("{}", "No jobs found.".dimmed());
        }
        return Ok(());
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&jobs)?);
        return Ok(());
    }

    // Table header
    println!(
        "{:<12} {:<10} {:<12} {:<40} {}",
        "ID".bold(),
        "ACTION".bold(),
        "STATUS".bold(),
        "PROMPT".bold(),
        "CREATED".bold()
    );
    println!("{}", "-".repeat(90));

    for job in jobs {
        let status_colored = match job.status_name() {
            "completed" => "completed".green().to_string(),
            "failed" => "failed".red().to_string(),
            "running" => "running".yellow().to_string(),
            "queued" => "queued".blue().to_string(),
            "cancelled" => "cancelled".dimmed().to_string(),
            s => s.to_string(),
        };

        let created = job.created_at.format("%Y-%m-%d %H:%M").to_string();

        println!(
            "{:<12} {:<10} {:<12} {:<40} {}",
            job.id,
            job.action.to_string(),
            status_colored,
            job.prompt_preview(38),
            created.dimmed()
        );
    }

    let count = db.count_jobs()?;
    if count as u32 > limit {
        println!();
        println!(
            "{}",
            format!("Showing {} of {} jobs. Use --limit to see more.", limit, count).dimmed()
        );
    }

    Ok(())
}

fn show_job(job_id: &str, format: &str, db: &Database) -> Result<()> {
    let job = db.get_job(job_id)?;

    match job {
        Some(job) => {
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else {
                println!();
                println!("{}: {}", "Job ID".cyan().bold(), job.id);
                println!("{}: {}", "Action".cyan().bold(), job.action);
                println!("{}: {}", "Status".cyan().bold(), job.status);
                println!("{}: {}", "Model".cyan().bold(), job.model);
                println!("{}: {}", "Created".cyan().bold(), job.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("{}: {}", "Updated".cyan().bold(), job.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
                println!();
                println!("{}:", "Prompt".cyan().bold());
                println!("  {}", job.params.prompt);
                println!();
                println!("{}:", "Parameters".cyan().bold());
                println!("  Aspect Ratio: {}", job.params.aspect_ratio);
                println!("  Size: {}", job.params.size);
                if let Some(seed) = job.params.seed {
                    println!("  Seed: {}", seed);
                }
                if let Some(neg) = &job.params.negative_prompt {
                    println!("  Negative: {}", neg);
                }

                if !job.images.is_empty() {
                    println!();
                    println!("{}:", "Images".cyan().bold());
                    for img in &job.images {
                        if let Some(path) = &img.path {
                            println!("  [{}] {}", img.index, path);
                        } else {
                            println!("  [{}] (base64 data, not downloaded)", img.index);
                        }
                    }
                }

                if let Some(parent) = &job.parent_id {
                    println!();
                    println!("{}: {}", "Parent Job".cyan().bold(), parent);
                }
            }
        }
        None => {
            if format == "json" {
                println!("null");
            } else {
                eprintln!("{}: Job '{}' not found", "Error".red().bold(), job_id);
            }
        }
    }

    Ok(())
}

fn delete_job(job_id: &str, db: &Database) -> Result<()> {
    if db.delete_job(job_id)? {
        println!("{} Deleted job: {}", "✓".green(), job_id);
    } else {
        eprintln!("{}: Job '{}' not found", "Error".red().bold(), job_id);
    }
    Ok(())
}

fn clear_jobs(force: bool, db: &Database) -> Result<()> {
    let count = db.count_jobs()?;

    if count == 0 {
        println!("{}", "No jobs to clear.".dimmed());
        return Ok(());
    }

    if !force {
        eprintln!(
            "{}: This will delete {} job(s). Use --force to confirm.",
            "Warning".yellow().bold(),
            count
        );
        return Ok(());
    }

    // Delete all jobs by listing and deleting each
    let jobs = db.list_jobs(count as u32 + 1, None)?;
    for job in jobs {
        db.delete_job(&job.id)?;
    }

    println!("{} Cleared {} job(s)", "✓".green(), count);
    Ok(())
}
