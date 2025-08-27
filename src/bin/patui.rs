//! Pali TUI - Terminal user interface for Pali todo management

#[cfg(not(feature = "tui"))]
compile_error!("The 'tui' feature must be enabled to build patui");

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use pali_terminal::tui::{app::App, ui};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;
    
    // Run the TUI
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {err:?}");
    }

    Ok(())
}

#[allow(clippy::unused_async)] // Function is async for future extensibility 
async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                _ => {
                    app.handle_key(key.code)?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

// Fallback main for when tui feature is disabled
#[cfg(not(feature = "tui"))]
fn main() {
    eprintln!("patui was built without TUI support. Enable the 'tui' feature to use this binary.");
    std::process::exit(1);
}