use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::path::PathBuf;

use super::app::{App, AppMode, SettingsField};
use crate::api::GeminiClient;
use crate::core::{GenerateParams, Job};

/// Handle input in main mode
pub async fn handle_main_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Home => app.selected_job = 0,
        KeyCode::End => {
            if !app.jobs.is_empty() {
                app.selected_job = app.jobs.len() - 1;
            }
        }

        // Enter input mode
        KeyCode::Char('i') | KeyCode::Char('/') => {
            app.mode = AppMode::Input;
            app.clear_messages();
        }

        // View job details
        KeyCode::Enter => {
            if let Some(job) = app.selected_job().cloned() {
                app.current_job = Some(job);
                app.mode = AppMode::JobDetail;
            }
        }

        // Open settings
        KeyCode::Char('s') => {
            app.mode = AppMode::Settings;
            app.settings_selected = 0;
            app.settings_editing = false;
        }

        // Refresh
        KeyCode::Char('r') => {
            app.load_jobs()?;
            app.set_status("Refreshed job list");
        }

        // Delete job
        KeyCode::Char('d') => {
            if let Some(job) = app.selected_job() {
                let id = job.id.clone();
                app.db.delete_job(&id)?;
                app.load_jobs()?;
                app.set_status(format!("Deleted job: {}", id));
            }
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }

        _ => {}
    }
    Ok(())
}

/// Handle input in text input mode
pub async fn handle_input_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.mode = AppMode::Main;
            app.input.clear();
            app.cursor_pos = 0;
        }

        KeyCode::Enter => {
            if !app.input.is_empty() {
                let prompt = app.input.clone();
                app.input.clear();
                app.cursor_pos = 0;
                app.mode = AppMode::Main;

                // Generate image
                generate_image(app, prompt).await?;
            }
        }

        KeyCode::Char(c) => {
            app.input.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
        }

        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
                app.input.remove(app.cursor_pos);
            }
        }

        KeyCode::Delete => {
            if app.cursor_pos < app.input.len() {
                app.input.remove(app.cursor_pos);
            }
        }

        KeyCode::Left => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }

        KeyCode::Right => {
            if app.cursor_pos < app.input.len() {
                app.cursor_pos += 1;
            }
        }

        KeyCode::Home => {
            app.cursor_pos = 0;
        }

        KeyCode::End => {
            app.cursor_pos = app.input.len();
        }

        _ => {}
    }
    Ok(())
}

/// Handle input in job detail mode
pub fn handle_job_detail_input(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
            app.mode = AppMode::Main;
            app.current_job = None;
        }

        // Could add download, re-run, etc.
        _ => {}
    }
    Ok(())
}

/// Handle input in settings mode
pub fn handle_settings_input(app: &mut App, key: KeyEvent) -> Result<()> {
    let fields = SettingsField::all();

    if app.settings_editing {
        // Editing a text field
        match key.code {
            KeyCode::Esc => {
                app.settings_editing = false;
                app.settings_edit_buffer.clear();
            }

            KeyCode::Enter => {
                let field = fields[app.settings_selected];
                let value = app.settings_edit_buffer.clone();
                if let Err(e) = app.set_settings_value(&field, &value) {
                    app.set_error(e.to_string());
                } else {
                    app.set_status(format!("Updated {}", field.label()));
                }
                app.settings_editing = false;
                app.settings_edit_buffer.clear();
            }

            KeyCode::Char(c) => {
                app.settings_edit_buffer.push(c);
            }

            KeyCode::Backspace => {
                app.settings_edit_buffer.pop();
            }

            _ => {}
        }
    } else {
        // Navigation
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.settings_selected > 0 {
                    app.settings_selected -= 1;
                }
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if app.settings_selected < fields.len() - 1 {
                    app.settings_selected += 1;
                }
            }

            KeyCode::Enter | KeyCode::Char(' ') => {
                let field = &fields[app.settings_selected];

                // Check if this field has options to cycle
                if app.get_settings_options(field).is_some() {
                    app.cycle_settings_option(field)?;
                    app.set_status(format!("Updated {}", field.label()));
                } else {
                    // Enter edit mode for text fields
                    app.settings_editing = true;
                    app.settings_edit_buffer = app.get_settings_value(field);
                }
            }

            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Main;
                app.clear_messages();
            }

            _ => {}
        }
    }
    Ok(())
}

/// Generate an image from a prompt
async fn generate_image(app: &mut App, prompt: String) -> Result<()> {
    app.set_status(format!("Generating: {}...", &prompt));
    app.generating = true;

    // Build parameters from config
    let params = GenerateParams::new(&prompt)
        .with_aspect_ratio(&app.config.defaults.aspect_ratio)
        .with_size(&app.config.defaults.size)
        .with_model(&app.config.api.model);

    // Create job
    let mut job = Job::new_generate(params);
    app.db.insert_job(&job)?;

    // Create client
    let client = match GeminiClient::from_config(&app.config) {
        Ok(c) => c,
        Err(e) => {
            job.set_failed(e.to_string());
            app.db.update_job(&job)?;
            app.load_jobs()?;
            app.set_error(e.to_string());
            app.generating = false;
            return Ok(());
        }
    };

    // Set running
    job.set_running(0);
    app.db.update_job(&job)?;

    // Generate
    match client.generate(&job.params).await {
        Ok(response) => {
            if let Err(e) = client.process_response(&mut job, response) {
                job.set_failed(e.to_string());
                app.db.update_job(&job)?;
                app.load_jobs()?;
                app.set_error(e.to_string());
                app.generating = false;
                return Ok(());
            }

            // Download if enabled
            if app.config.output.auto_download {
                let output_dir = PathBuf::from(&app.config.output.directory);
                match client.download_images(&mut job, &output_dir).await {
                    Ok(paths) => {
                        app.set_status(format!(
                            "Generated {} image(s): {}",
                            paths.len(),
                            paths.first().unwrap_or(&String::new())
                        ));
                    }
                    Err(e) => {
                        app.set_error(format!("Download failed: {}", e));
                    }
                }
            } else {
                app.set_status(format!("Generated {} image(s)", job.images.len()));
            }
        }
        Err(e) => {
            job.set_failed(e.to_string());
            app.set_error(e.to_string());
        }
    }

    app.db.update_job(&job)?;
    app.load_jobs()?;
    app.generating = false;

    Ok(())
}
