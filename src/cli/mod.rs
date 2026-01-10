pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "banana",
    author = "Christian Weinmayr",
    version,
    about = "🍌 Nano Banana Pro CLI - Generate images with Google Gemini",
    long_about = r#"🍌 Nano Banana Pro CLI - Generate images with Google Gemini

A powerful CLI for generating and editing images using Google's Gemini AI models.
Run without arguments to launch the interactive TUI.

SETUP:
  Set your API key via environment variable or config:
    export GEMINI_API_KEY=your-key-here
    banana config set api.key your-key-here

EXAMPLES:
  Generate an image:
    banana generate "a cosmic banana floating in space"
    banana g "sunset over mountains" --ar 16:9
    banana generate "minimalist logo" --size 2K --format json

  Edit an existing image:
    banana edit image.png "add a rainbow in the sky"
    banana e photo.jpg "make it look like a watercolor painting"

  View job history:
    banana jobs
    banana jobs show bn_abc12345
    banana jobs --status completed --limit 10

  Manage configuration:
    banana config show
    banana config set defaults.aspect_ratio 16:9
    banana config set api.model gemini-3-pro-image-preview

  Launch interactive TUI:
    banana

OUTPUT FORMATS:
  --format text   Human-readable output (default)
  --format json   Machine-readable JSON for AI agents
  --format quiet  Minimal output, just file paths

For AI agent integration, use --format json for structured output."#,
    after_help = r#"CONFIGURATION:
  Config file: ~/.config/banana/config.toml (macOS/Linux)
  Database: ~/.local/share/banana-cli/jobs.db

  Available models:
    - gemini-3-pro-image-preview (default)
    - gemini-2.5-flash-image (fast)
    - imagen-4.0-generate-001 (high quality)

  Aspect ratios: 1:1, 2:3, 3:2, 3:4, 4:3, 4:5, 5:4, 9:16, 16:9, 21:9
  Sizes: 1K (default), 2K, 4K (4K requires Gemini 3 Pro)

MORE INFO:
  GitHub: https://github.com/christianweinmayr/nanobanan-cli"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a new image from a text prompt
    ///
    /// Creates images using Google's Gemini AI models from your text description.
    /// Images are saved to the configured output directory by default.
    #[command(
        alias = "g",
        after_help = r#"EXAMPLES:
  Basic generation:
    banana generate "a red apple on a wooden table"

  With aspect ratio:
    banana generate "panoramic mountain landscape" --ar 21:9

  High resolution:
    banana generate "detailed portrait" --size 2K

  JSON output for AI agents:
    banana generate "abstract art" --format json

  Custom output directory:
    banana generate "logo design" --output ./logos"#
    )]
    Generate(commands::generate::GenerateArgs),

    /// Edit an existing image using a text prompt
    ///
    /// Modifies images using semantic editing - describe what you want to change
    /// and the AI will apply the edit while preserving the rest of the image.
    #[command(
        alias = "e",
        after_help = r#"EXAMPLES:
  Add elements:
    banana edit photo.png "add sunglasses to the person"

  Change style:
    banana edit image.jpg "convert to pencil sketch style"

  Modify colors:
    banana edit scene.png "change the sky to sunset colors"

  Remove elements:
    banana edit room.jpg "remove the chair in the corner""#
    )]
    Edit(commands::edit::EditArgs),

    /// Manage and view job history
    ///
    /// View, inspect, and manage your generation history.
    /// All jobs are persisted in a local SQLite database.
    #[command(
        alias = "j",
        after_help = r#"EXAMPLES:
  List recent jobs:
    banana jobs
    banana jobs --limit 50

  Filter by status:
    banana jobs --status completed
    banana jobs --status failed

  View job details:
    banana jobs show bn_abc12345

  Delete a job:
    banana jobs delete bn_abc12345

  Clear all history:
    banana jobs clear --force

  JSON output:
    banana jobs --format json"#
    )]
    Jobs(commands::jobs::JobsArgs),

    /// View or modify configuration
    ///
    /// Manage API keys, default parameters, and output settings.
    /// Changes are saved to the config file immediately.
    #[command(
        alias = "c",
        after_help = r#"EXAMPLES:
  Show all settings:
    banana config show

  Get a specific value:
    banana config get defaults.aspect_ratio

  Set values:
    banana config set api.key YOUR_API_KEY
    banana config set defaults.aspect_ratio 16:9
    banana config set defaults.size 2K
    banana config set output.directory ~/Pictures/banana

  Show config file path:
    banana config path

  Reset to defaults:
    banana config reset --force

AVAILABLE SETTINGS:
  api.key              - Gemini API key
  api.model            - Default model
  defaults.aspect_ratio - Default aspect ratio
  defaults.size        - Default image size (1K, 2K, 4K)
  output.directory     - Where to save images
  output.auto_download - Auto-download images (true/false)
  output.display       - Display mode (terminal/viewer/none)
  tui.show_images      - Show images in TUI (true/false)
  tui.theme            - TUI theme (dark/light)"#
    )]
    Config(commands::config::ConfigArgs),
}
