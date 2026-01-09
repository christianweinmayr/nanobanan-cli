mod app;
mod event_handler;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

use crate::config::Config;
use crate::db::Database;

pub use app::{App, AppMode};

/// Run the TUI application
pub async fn run(config: &mut Config, db: &Database) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(config.clone(), db.clone());
    app.load_jobs()?;

    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Save config if changed
    if app.config_changed {
        *config = app.config.clone();
        config.save()?;
    }

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                // Global quit shortcuts
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }
                if key.code == KeyCode::Char('q') && app.mode != AppMode::Input && app.mode != AppMode::Settings {
                    return Ok(());
                }

                // Handle mode-specific input
                match app.mode {
                    AppMode::Main => event_handler::handle_main_input(app, key).await?,
                    AppMode::Input => event_handler::handle_input_mode(app, key).await?,
                    AppMode::JobDetail => event_handler::handle_job_detail_input(app, key)?,
                    AppMode::Settings => event_handler::handle_settings_input(app, key)?,
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            return Ok(());
        }
    }
}
