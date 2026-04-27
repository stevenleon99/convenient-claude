use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use super::app::App;

/// Handle key events. Returns true if the app should continue running.
pub fn handle_events(app: &mut App, project_dir: &std::path::Path, workspace_root: &std::path::Path) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            // Only handle key press events (ignore release/repeat on Windows)
            if key.kind != KeyEventKind::Press {
                return Ok(true);
            }

            // Confirmation dialog takes priority
            if app.confirm_quit {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.confirm_and_install(project_dir, workspace_root);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') | KeyCode::Esc => {
                        app.confirm_without_install();
                    }
                    KeyCode::Char('c') => {
                        app.cancel_quit();
                    }
                    _ => {}
                }
                return Ok(!app.should_quit);
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.quit();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous();
                }
                KeyCode::Tab => {
                    app.next_tab();
                    app.refresh(project_dir, workspace_root);
                }
                KeyCode::BackTab => {
                    app.prev_tab();
                    app.refresh(project_dir, workspace_root);
                }
                KeyCode::Enter => {
                    app.toggle_detail();
                }
                KeyCode::Char('r') => {
                    app.refresh(project_dir, workspace_root);
                }
                KeyCode::Char(' ') => {
                    app.toggle_selected();
                }
                KeyCode::Char('a') => {
                    app.select_all();
                }
                KeyCode::Char('c') => {
                    app.clear_selection();
                }
                KeyCode::Char('i') => {
                    app.install_selected(project_dir, workspace_root);
                    app.refresh(project_dir, workspace_root);
                }
                KeyCode::Char('I') => {
                    app.install_all_selected(project_dir, workspace_root);
                    app.refresh(project_dir, workspace_root);
                }
                _ => {}
            }
        }
    }
    Ok(!app.should_quit)
}
