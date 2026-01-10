# Nano Banana Pro CLI

A powerful command-line tool for generating and editing images using Google's Gemini AI models.

## Features

- **Image Generation** - Create images from text prompts
- **Image Editing** - Modify existing images with natural language
- **Interactive TUI** - Full terminal user interface with image preview
- **Job History** - SQLite-backed persistence of all generations
- **Multiple Output Formats** - Human-readable, JSON for AI agents, or quiet mode
- **Configurable** - Aspect ratios, sizes, models, and output settings

## Installation

### From Source

```bash
git clone https://github.com/christianweinmayr/nanobanan-cli.git
cd nanobanan-cli
cargo build --release
```

The binary will be at `./target/release/banana`.

### Add to PATH (optional)

```bash
cp ./target/release/banana ~/.local/bin/
# or
sudo cp ./target/release/banana /usr/local/bin/
```

## Setup

Set your Gemini API key via environment variable or config:

```bash
# Environment variable (recommended)
export GEMINI_API_KEY=your-key-here

# Or save to config
banana config set api.key your-key-here
```

Get your API key from [Google AI Studio](https://aistudio.google.com/apikey).

## Usage

### Generate Images

```bash
# Basic generation
banana generate "a cosmic banana floating in space"

# Short alias
banana g "sunset over mountains"

# With aspect ratio
banana g "panoramic landscape" --ar 16:9
banana g "phone wallpaper" --ar 9:16

# High resolution
banana generate "detailed portrait" --size 2K

# JSON output for AI agents
banana generate "abstract art" --format json

# Custom output directory
banana generate "logo design" --output ./logos
```

### Edit Images

```bash
# Add elements
banana edit photo.png "add sunglasses to the person"

# Change style
banana edit image.jpg "convert to pencil sketch style"

# Modify colors
banana edit scene.png "change the sky to sunset colors"
```

### View Job History

```bash
# List recent jobs
banana jobs

# Filter by status
banana jobs --status completed
banana jobs --status failed

# View job details
banana jobs show bn_abc12345

# JSON output
banana jobs --format json
```

### Configuration

```bash
# Show all settings
banana config show

# Get a specific value
banana config get defaults.aspect_ratio

# Set values
banana config set defaults.aspect_ratio 16:9
banana config set defaults.size 2K
banana config set output.directory ~/Pictures/banana

# Show config file path
banana config path
```

### Interactive TUI

Launch the terminal UI by running `banana` without arguments:

```bash
banana
```

## Available Models

| Model | Description |
|-------|-------------|
| `gemini-3-pro-image-preview` | Default model |
| `gemini-2.5-flash-image` | Fast generation |
| `imagen-4.0-generate-001` | High quality |

## Aspect Ratios

`1:1`, `2:3`, `3:2`, `3:4`, `4:3`, `4:5`, `5:4`, `9:16`, `16:9`, `21:9`

## Sizes

| Size | Description |
|------|-------------|
| `1K` | Default |
| `2K` | Higher resolution |
| `4K` | Maximum (requires Gemini 3 Pro) |

## Configuration File

Located at:
- macOS/Linux: `~/.config/banana/config.toml`

Example config:

```toml
[api]
key = "your-api-key"
model = "gemini-3-pro-image-preview"

[defaults]
aspect_ratio = "1:1"
size = "1K"

[output]
directory = "./banana-output"
auto_download = true
display = "terminal"

[tui]
show_images = true
theme = "dark"
```

## Output Formats

| Format | Description |
|--------|-------------|
| `text` | Human-readable output (default) |
| `json` | Machine-readable JSON for AI agents |
| `quiet` | Minimal output, just file paths |

## AI Agent Integration

Use `--format json` for structured output suitable for AI agents:

```bash
banana generate "minimalist logo" --format json
```

## License

MIT

## Author

Christian Weinmayr
