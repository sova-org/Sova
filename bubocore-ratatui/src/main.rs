#[macro_use]
extern crate rocket;
use app::{App, CurrentScreen};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::io::stderr;
use std::{error::Error, io};
use ui::ui;

mod app;
mod ui;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let _res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let _ = rocket::build().mount("/", routes![index]);
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            // Matching depending on which screen we are viewing
            match app.current_screen {
                CurrentScreen::Editor => match key.code {
                    KeyCode::F(1) => {
                        app.current_screen = CurrentScreen::Editor;
                    }
                    KeyCode::F(2) => {
                        app.current_screen = CurrentScreen::Grid;
                    }
                    KeyCode::F(3) => {
                        app.current_screen = CurrentScreen::Options;
                    }
                    KeyCode::Tab => {
                        app.current_screen = CurrentScreen::Grid;
                    }
                    _ => {}
                },
                CurrentScreen::Grid => match key.code {
                    KeyCode::F(1) => {
                        app.current_screen = CurrentScreen::Editor;
                    }
                    KeyCode::F(2) => {
                        app.current_screen = CurrentScreen::Grid;
                    }
                    KeyCode::F(3) => {
                        app.current_screen = CurrentScreen::Options;
                    }
                    KeyCode::Tab => {
                        app.current_screen = CurrentScreen::Options;
                    }
                    _ => {}
                },
                CurrentScreen::Options => match key.code {
                    KeyCode::F(1) => {
                        app.current_screen = CurrentScreen::Editor;
                    }
                    KeyCode::F(2) => {
                        app.current_screen = CurrentScreen::Grid;
                    }
                    KeyCode::F(3) => {
                        app.current_screen = CurrentScreen::Options;
                    }
                    KeyCode::Tab => {
                        app.current_screen = CurrentScreen::Editor;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
