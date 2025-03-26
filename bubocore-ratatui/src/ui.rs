use ratatui::Frame;

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &App) {
    // TODO: implement drawing logic using the different CurrentScreen modes

    match app.current_screen {
        CurrentScreen::Editor => {
            // TODO: draw editor frames
        }
        CurrentScreen::Grid => {
            // TODO: draw grid frames
        }
        CurrentScreen::Options => {
            // TODO: draw options frames
        }
    }
}
