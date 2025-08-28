//! Pali TUI - Terminal user interface for Pali todo management

#[cfg(not(feature = "tui"))]
compile_error!("The 'tui' feature must be enabled to build patui");

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pali_terminal::tui::{app::App, ui};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

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
        eprintln!("Error: {err}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    // Load initial todos
    app.load_todos().await?;

    let tick_rate = Duration::from_millis(250); // 4 FPS for spinner animation
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events with timeout for spinner animation
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C globally for quit confirmation
                if key.code == crossterm::event::KeyCode::Char('c')
                    && key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    app.handle_ctrl_c();
                } else {
                    app.handle_key(key.code).await?;
                }
            }
        }

        // Update spinner animation and message timers
        if last_tick.elapsed() >= tick_rate {
            app.tick_spinner();
            app.tick_messages();
            last_tick = Instant::now();
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
