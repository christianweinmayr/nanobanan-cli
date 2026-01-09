use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub tui: TuiConfig,

    #[serde(skip)]
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,
    #[serde(default = "default_size")]
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_directory")]
    pub directory: String,
    #[serde(default = "default_true")]
    pub auto_download: bool,
    #[serde(default = "default_display")]
    pub display: DisplayMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_true")]
    pub show_images: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    #[default]
    Terminal,
    Viewer,
    None,
}

impl DisplayMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayMode::Terminal => "terminal",
            DisplayMode::Viewer => "viewer",
            DisplayMode::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "terminal" => DisplayMode::Terminal,
            "viewer" => DisplayMode::Viewer,
            "none" => DisplayMode::None,
            _ => DisplayMode::Terminal,
        }
    }

    pub fn variants() -> &'static [&'static str] {
        &["terminal", "viewer", "none"]
    }
}

// Default value functions
fn default_model() -> String {
    "gemini-3-pro-image-preview".to_string()
}

fn default_base_url() -> String {
    "https://generativelanguage.googleapis.com/v1beta".to_string()
}

fn default_aspect_ratio() -> String {
    "1:1".to_string()
}

fn default_size() -> String {
    "1K".to_string()
}

fn default_output_directory() -> String {
    "./banana-output".to_string()
}

fn default_true() -> bool {
    true
}

fn default_display() -> DisplayMode {
    DisplayMode::Terminal
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            key: None,
            model: default_model(),
            base_url: default_base_url(),
        }
    }
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: default_aspect_ratio(),
            size: default_size(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            directory: default_output_directory(),
            auto_download: true,
            display: DisplayMode::Terminal,
        }
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            show_images: true,
            theme: default_theme(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            defaults: DefaultsConfig::default(),
            output: OutputConfig::default(),
            tui: TuiConfig::default(),
            config_path: PathBuf::new(),
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "nanobanan", "banana-cli")
            .context("Failed to determine config directory")?;
        Ok(proj_dirs.config_dir().to_path_buf())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load config from file or create default
    pub fn load_or_create() -> Result<Self> {
        let config_path = Self::config_path()?;

        // Check for API key in environment first
        let env_key = std::env::var("GEMINI_API_KEY").ok();

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let mut config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            config.config_path = config_path;

            // Environment variable takes precedence
            if let Some(key) = env_key {
                config.api.key = Some(key);
            }

            Ok(config)
        } else {
            let mut config = Config::default();
            config.config_path = config_path;

            // Use environment variable if available
            if let Some(key) = env_key {
                config.api.key = Some(key);
            }

            // Create config directory and save default config
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&self.config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get API key (from config or environment)
    pub fn api_key(&self) -> Option<&str> {
        self.api.key.as_deref()
    }

    /// Set a config value by key path (e.g., "api.key", "defaults.aspect_ratio")
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "api.key" => self.api.key = Some(value.to_string()),
            "api.model" => self.api.model = value.to_string(),
            "api.base_url" => self.api.base_url = value.to_string(),
            "defaults.aspect_ratio" => {
                // Validate aspect ratio
                let valid = ["1:1", "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9"];
                if valid.contains(&value) {
                    self.defaults.aspect_ratio = value.to_string();
                } else {
                    anyhow::bail!("Invalid aspect ratio. Valid values: {}", valid.join(", "));
                }
            }
            "defaults.size" => {
                let valid = ["1K", "2K", "4K"];
                if valid.contains(&value) {
                    self.defaults.size = value.to_string();
                } else {
                    anyhow::bail!("Invalid size. Valid values: {}", valid.join(", "));
                }
            }
            "output.directory" => self.output.directory = value.to_string(),
            "output.auto_download" => {
                self.output.auto_download = value.parse()
                    .context("Invalid boolean value")?;
            }
            "output.display" => {
                self.output.display = DisplayMode::from_str(value);
            }
            "tui.show_images" => {
                self.tui.show_images = value.parse()
                    .context("Invalid boolean value")?;
            }
            "tui.theme" => self.tui.theme = value.to_string(),
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
        Ok(())
    }

    /// Get a config value by key path
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "api.key" => self.api.key.clone().map(|_| "****".to_string()), // Mask API key
            "api.model" => Some(self.api.model.clone()),
            "api.base_url" => Some(self.api.base_url.clone()),
            "defaults.aspect_ratio" => Some(self.defaults.aspect_ratio.clone()),
            "defaults.size" => Some(self.defaults.size.clone()),
            "output.directory" => Some(self.output.directory.clone()),
            "output.auto_download" => Some(self.output.auto_download.to_string()),
            "output.display" => Some(self.output.display.as_str().to_string()),
            "tui.show_images" => Some(self.tui.show_images.to_string()),
            "tui.theme" => Some(self.tui.theme.clone()),
            _ => None,
        }
    }

    /// Get all config keys
    pub fn keys() -> &'static [&'static str] {
        &[
            "api.key",
            "api.model",
            "api.base_url",
            "defaults.aspect_ratio",
            "defaults.size",
            "output.directory",
            "output.auto_download",
            "output.display",
            "tui.show_images",
            "tui.theme",
        ]
    }

    /// Available aspect ratios
    pub fn aspect_ratios() -> &'static [&'static str] {
        &["1:1", "2:3", "3:2", "3:4", "4:3", "4:5", "5:4", "9:16", "16:9", "21:9"]
    }

    /// Available sizes
    pub fn sizes() -> &'static [&'static str] {
        &["1K", "2K", "4K"]
    }

    /// Available models
    pub fn models() -> &'static [&'static str] {
        &[
            "gemini-3-pro-image-preview",
            "gemini-2.5-flash-image",
            "imagen-4.0-generate-001",
        ]
    }
}
