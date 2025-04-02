use crate::app::App;
use clap::{Parser, arg, command};
use color_eyre::Result;
use names::Generator;
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
mod components;
mod app;
mod event;
mod link;
mod network;
mod ui;
mod commands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
    #[arg(short, long)]
    username: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let username = match args.username {
        Some(name) => name,
        None => {
            let mut generator = Generator::default();
            generator.next().unwrap_or_else(|| "BuboUser".to_string())
        }
    };

    let terminal = init_terminal()?;
    let mut app = App::new(args.ip, args.port, username);
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
