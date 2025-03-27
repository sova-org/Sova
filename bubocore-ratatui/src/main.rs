use crate::app::App;
use color_eyre::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    },
};
use std::io;

mod app;
mod components;
mod event;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    // Panic handler utilisé par démos Ratatui
    color_eyre::install()?;
    let terminal = init_terminal()?;
    let mut app = App::new();
    let result = app.run(terminal).await;
    restore_terminal()?;
    result?;
    Ok(())
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stderr>>> {
    terminal::enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    terminal::disable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
