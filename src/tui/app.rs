use crate::config::Config;
use crate::core::Job;
use crate::db::Database;
use anyhow::Result;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Main view with job list
    Main,
    /// Text input mode
    Input,
    /// Viewing job details
    JobDetail,
    /// Settings screen
    Settings,
}

/// Settings field being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Model,
    AspectRatio,
    Size,
    OutputDirectory,
    AutoDownload,
    Display,
    ShowImages,
    Theme,
}

impl SettingsField {
    pub fn all() -> &'static [SettingsField] {
        &[
            SettingsField::Model,
            SettingsField::AspectRatio,
            SettingsField::Size,
            SettingsField::OutputDirectory,
            SettingsField::AutoDownload,
            SettingsField::Display,
            SettingsField::ShowImages,
            SettingsField::Theme,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsField::Model => "Model",
            SettingsField::AspectRatio => "Aspect Ratio",
            SettingsField::Size => "Size",
            SettingsField::OutputDirectory => "Output Directory",
            SettingsField::AutoDownload => "Auto Download",
            SettingsField::Display => "Display Mode",
            SettingsField::ShowImages => "Show Images in TUI",
            SettingsField::Theme => "Theme",
        }
    }

    pub fn config_key(&self) -> &'static str {
        match self {
            SettingsField::Model => "api.model",
            SettingsField::AspectRatio => "defaults.aspect_ratio",
            SettingsField::Size => "defaults.size",
            SettingsField::OutputDirectory => "output.directory",
            SettingsField::AutoDownload => "output.auto_download",
            SettingsField::Display => "output.display",
            SettingsField::ShowImages => "tui.show_images",
            SettingsField::Theme => "tui.theme",
        }
    }
}

/// TUI application state
pub struct App {
    /// Current mode
    pub mode: AppMode,

    /// Configuration
    pub config: Config,

    /// Database
    pub db: Database,

    /// Current prompt input
    pub input: String,

    /// Cursor position in input
    pub cursor_pos: usize,

    /// Job list
    pub jobs: Vec<Job>,

    /// Selected job index
    pub selected_job: usize,

    /// Currently viewing job (for detail view)
    pub current_job: Option<Job>,

    /// Status message
    pub status_message: Option<String>,

    /// Error message
    pub error_message: Option<String>,

    /// Whether to quit
    pub should_quit: bool,

    /// Whether config was changed
    pub config_changed: bool,

    /// Settings: selected field index
    pub settings_selected: usize,

    /// Settings: currently editing
    pub settings_editing: bool,

    /// Settings: edit buffer
    pub settings_edit_buffer: String,

    /// Generation in progress
    pub generating: bool,
}

impl App {
    pub fn new(config: Config, db: Database) -> Self {
        Self {
            mode: AppMode::Main,
            config,
            db,
            input: String::new(),
            cursor_pos: 0,
            jobs: Vec::new(),
            selected_job: 0,
            current_job: None,
            status_message: None,
            error_message: None,
            should_quit: false,
            config_changed: false,
            settings_selected: 0,
            settings_editing: false,
            settings_edit_buffer: String::new(),
        generating: false,
        }
    }

    /// Load jobs from database
    pub fn load_jobs(&mut self) -> Result<()> {
        self.jobs = self.db.list_jobs(50, None)?;
        if self.selected_job >= self.jobs.len() && !self.jobs.is_empty() {
            self.selected_job = self.jobs.len() - 1;
        }
        Ok(())
    }

    /// Set status message
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.error_message = None;
    }

    /// Set error message
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.status_message = None;
    }

    /// Clear messages
    pub fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
    }

    /// Get the currently selected job
    pub fn selected_job(&self) -> Option<&Job> {
        self.jobs.get(self.selected_job)
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_job > 0 {
            self.selected_job -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_job < self.jobs.len().saturating_sub(1) {
            self.selected_job += 1;
        }
    }

    /// Get current settings value
    pub fn get_settings_value(&self, field: &SettingsField) -> String {
        match field {
            SettingsField::Model => self.config.api.model.clone(),
            SettingsField::AspectRatio => self.config.defaults.aspect_ratio.clone(),
            SettingsField::Size => self.config.defaults.size.clone(),
            SettingsField::OutputDirectory => self.config.output.directory.clone(),
            SettingsField::AutoDownload => self.config.output.auto_download.to_string(),
            SettingsField::Display => self.config.output.display.as_str().to_string(),
            SettingsField::ShowImages => self.config.tui.show_images.to_string(),
            SettingsField::Theme => self.config.tui.theme.clone(),
        }
    }

    /// Set settings value
    pub fn set_settings_value(&mut self, field: &SettingsField, value: &str) -> Result<()> {
        self.config.set(field.config_key(), value)?;
        self.config_changed = true;
        Ok(())
    }

    /// Get options for a settings field (if applicable)
    pub fn get_settings_options(&self, field: &SettingsField) -> Option<Vec<&'static str>> {
        match field {
            SettingsField::Model => Some(Config::models().to_vec()),
            SettingsField::AspectRatio => Some(Config::aspect_ratios().to_vec()),
            SettingsField::Size => Some(Config::sizes().to_vec()),
            SettingsField::AutoDownload => Some(vec!["true", "false"]),
            SettingsField::Display => Some(crate::config::DisplayMode::variants().to_vec()),
            SettingsField::ShowImages => Some(vec!["true", "false"]),
            SettingsField::Theme => Some(vec!["dark", "light"]),
            _ => None,
        }
    }

    /// Cycle to next option for a settings field
    pub fn cycle_settings_option(&mut self, field: &SettingsField) -> Result<()> {
        if let Some(options) = self.get_settings_options(field) {
            let current = self.get_settings_value(field);
            let current_idx = options.iter().position(|&o| o == current).unwrap_or(0);
            let next_idx = (current_idx + 1) % options.len();
            self.set_settings_value(field, options[next_idx])?;
        }
        Ok(())
    }
}
