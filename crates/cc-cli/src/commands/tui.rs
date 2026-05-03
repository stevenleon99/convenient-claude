use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::tui::app::App;
use crate::tui::event as tui_event;
use crate::tui::ui;

pub fn run(project_dir: &std::path::Path, app_dir: &std::path::Path) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Init app state
    let mut app = App::new();
    app.refresh(project_dir, app_dir);

    // Run app loop — restore terminal even on panic
    let res = run_app(&mut terminal, &mut app, project_dir, app_dir);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("TUI error: {err}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    project_dir: &std::path::Path,
    app_dir: &std::path::Path,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !tui_event::handle_events(app, project_dir, app_dir)? {
            return Ok(());
        }
    }
}
