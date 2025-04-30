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
mod app;
mod components;
mod disk;
mod event;
mod link;
mod network;
mod ui;

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

    // --- Load Client Config ---
    let client_config = match disk::read_client_config().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Could not load client config: {}. Using defaults.", e);
            // Optionally log the error in more detail or exit if config is critical
            disk::ClientConfig::default()
        }
    };
    // --- End Load Client Config ---

    let username = match args.username.or_else(|| client_config.last_username.clone()) { // Use config username if available
        Some(name) => name,
        None => {
            let mut generator = Generator::default();
            generator.next().unwrap_or_else(|| "BuboUser".to_string())
        }
    };

    // Use config IP and port if available, otherwise use CLI args or defaults
    let ip = if args.ip != "127.0.0.1" { // Check if user provided IP via CLI
        args.ip.clone()
    } else {
        client_config.last_ip_address.clone().unwrap_or(args.ip)
    };

    let port = if args.port != 8080 { // Check if user provided port via CLI
        args.port
    } else {
        client_config.last_port.unwrap_or(args.port)
    };

    let terminal = init_terminal()?;
    // Pass loaded config to App::new
    let mut app = App::new(ip, port, username, client_config); // Pass config here
    let run_result = app.run(terminal).await; // Store the result of run

    // --- Save Client Config ---
    // Update config with last used values before saving
    app.update_config_before_save(); // Add this method call
    if let Err(e) = disk::write_client_config(&app.client_config).await {
         eprintln!("Warning: Failed to save client config: {}", e);
         // Log or handle the error appropriately
    }
    // --- End Save Client Config ---

    restore_terminal()?;
    run_result?; // Check the result of app.run after restoring terminal and saving config
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
