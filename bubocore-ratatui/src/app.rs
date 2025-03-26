/// The current screen that the user is on:
/// - Editor: The user is editing a script.
/// - Grid: The user is browsing the grid of scripts.
/// - Options: The user is viewing the options menu.
pub enum CurrentScreen {
    Editor,
    Grid,
    Options,
}

pub struct App {
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Grid,
        }
    }
}
