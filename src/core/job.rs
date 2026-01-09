use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::params::GenerateParams;

/// Represents a single generated image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobImage {
    /// Index in the generation (0-3)
    pub index: u8,
    /// Base64 encoded image data (before download)
    pub data: Option<String>,
    /// Local file path (after download)
    pub path: Option<String>,
    /// Mime type
    pub mime_type: String,
}

/// The type of action performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum JobAction {
    /// Generate new image from prompt
    Generate,
    /// Edit existing image
    Edit {
        /// Path to source image
        source_image: String,
    },
}

impl std::fmt::Display for JobAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobAction::Generate => write!(f, "generate"),
            JobAction::Edit { .. } => write!(f, "edit"),
        }
    }
}

/// Status of a job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status")]
pub enum JobStatus {
    /// Job is queued
    Queued,
    /// Job is being processed
    Running {
        /// Progress percentage (0-100)
        progress: u8,
    },
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed {
        /// Error message
        error: String,
    },
    /// Job was cancelled
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "queued"),
            JobStatus::Running { progress } => write!(f, "running ({}%)", progress),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed { error } => write!(f, "failed: {}", error),
            JobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl JobStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Completed | JobStatus::Failed { .. } | JobStatus::Cancelled)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, JobStatus::Completed)
    }
}

/// A generation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job ID (e.g., "bn_abc12345")
    pub id: String,

    /// The action type
    pub action: JobAction,

    /// Generation parameters
    pub params: GenerateParams,

    /// Current status
    pub status: JobStatus,

    /// Generated images
    pub images: Vec<JobImage>,

    /// Model used
    pub model: String,

    /// When the job was created
    pub created_at: DateTime<Utc>,

    /// When the job was last updated
    pub updated_at: DateTime<Utc>,

    /// Parent job ID (for variations/edits)
    pub parent_id: Option<String>,
}

impl Job {
    /// Create a new generation job
    pub fn new_generate(params: GenerateParams) -> Self {
        let uuid = Uuid::new_v4();
        let id = format!("bn_{}", &uuid.to_string()[..8]);
        let now = Utc::now();

        Self {
            id,
            action: JobAction::Generate,
            model: params.model.clone(),
            params,
            status: JobStatus::Queued,
            images: Vec::new(),
            created_at: now,
            updated_at: now,
            parent_id: None,
        }
    }

    /// Create a new edit job
    pub fn new_edit(params: GenerateParams, source_image: String) -> Self {
        let uuid = Uuid::new_v4();
        let id = format!("bn_{}", &uuid.to_string()[..8]);
        let now = Utc::now();

        Self {
            id,
            action: JobAction::Edit { source_image },
            model: params.model.clone(),
            params,
            status: JobStatus::Queued,
            images: Vec::new(),
            created_at: now,
            updated_at: now,
            parent_id: None,
        }
    }

    /// Set job as running with progress
    pub fn set_running(&mut self, progress: u8) {
        self.status = JobStatus::Running { progress: progress.min(100) };
        self.updated_at = Utc::now();
    }

    /// Set job as completed
    pub fn set_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.updated_at = Utc::now();
    }

    /// Set job as failed
    pub fn set_failed(&mut self, error: impl Into<String>) {
        self.status = JobStatus::Failed { error: error.into() };
        self.updated_at = Utc::now();
    }

    /// Set job as cancelled
    pub fn set_cancelled(&mut self) {
        self.status = JobStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// Add an image to the job
    pub fn add_image(&mut self, index: u8, data: String, mime_type: String) {
        self.images.push(JobImage {
            index,
            data: Some(data),
            path: None,
            mime_type,
        });
        self.updated_at = Utc::now();
    }

    /// Get the prompt (truncated for display)
    pub fn prompt_preview(&self, max_len: usize) -> String {
        if self.params.prompt.len() <= max_len {
            self.params.prompt.clone()
        } else {
            format!("{}...", &self.params.prompt[..max_len.saturating_sub(3)])
        }
    }

    /// Get status as a simple string for filtering
    pub fn status_name(&self) -> &'static str {
        match &self.status {
            JobStatus::Queued => "queued",
            JobStatus::Running { .. } => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed { .. } => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }
}
