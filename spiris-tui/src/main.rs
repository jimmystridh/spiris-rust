//! Spiris TUI - Terminal User Interface for Spiris Bokföring API
//!
//! A comprehensive TUI application for managing customers, invoices, and articles
//! through the Spiris Bokföring och Fakturering API (formerly Visma eAccounting).
//!
//! ## Features
//!
//! - Full CRUD operations for customers, invoices, and articles
//! - Search and filtering capabilities
//! - Data export to CSV and JSON
//! - OAuth2 authentication
//! - Real-time validation
//! - Sorting and pagination
//!
//! ## Keyboard Shortcuts
//!
//! - `q`: Quit (when not in input mode)
//! - `Tab`/`Shift+Tab`: Navigate between screens
//! - `↑`/`↓`: Navigate lists
//! - `←`/`→`: Previous/Next page
//! - `n`: Create new entity (on customer/invoice/article screens)
//! - `e`: Edit current entity (on detail screens)
//! - `x`: Delete current entity (shows confirmation)
//! - `r`: Refresh data
//! - `s` or `/`: Open search
//! - `m`: Cycle search mode (on search screen)
//! - `d`: Dashboard
//! - `c`: Customers
//! - `i`: Invoices (when not in input mode)
//! - `a`: Articles (when not in input mode)
//! - `h` or `?`: Help
//! - `Esc`: Go back/Cancel
//! - `Enter`: Confirm/Open detail view

mod app;
mod auth;
mod config;
mod screens;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self as terminal_event, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
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
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle refresh if needed
        app.refresh_if_needed().await?;

        // Poll for events
        if terminal_event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = terminal_event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') if app.can_quit() => return Ok(()),
                        KeyCode::Esc => app.handle_escape(),
                        KeyCode::Enter => app.handle_enter().await?,
                        KeyCode::Tab => app.next_screen(),
                        KeyCode::BackTab => app.previous_screen(),
                        KeyCode::Up => app.handle_up(),
                        KeyCode::Down => app.handle_down(),
                        KeyCode::Left => app.handle_left(),
                        KeyCode::Right => app.handle_right(),
                        KeyCode::Char(c) => app.handle_char(c),
                        KeyCode::Backspace => app.handle_backspace(),
                        _ => {}
                    }
                }
            }
        }

        // Update app state (message timers, etc.)
        app.tick();
    }
}
