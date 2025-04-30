//! Components module for BuboCoreTUI
//!
//! This module contains all the UI components that make up the BuboCoreTUI interface.
//! Each component is responsible for a specific part of the UI and implements the
//! [`Component`] trait to ensure consistent behavior across the application.
//!
//! # Components
//!
//! * [`command_palette`] - Command palette for quick actions
//! * [`devices`] - Device management interface
//! * [`editor`] - Code editor with syntax highlighting
//! * [`grid`] - Scene grid visualization
//! * [`help`] - Help documentation viewer
//! * [`logs`] - Application log viewer
//! * [`navigation`] - Navigation menu and controls
//! * [`options`] - Application settings interface
//! * [`saveload`] - Save and load functionality
//! * [`splash`] - Splash screen and connection interface
//!
//! # Usage
//!
//! Components are typically used through the [`App`] struct, which manages the
//! application state and coordinates between different components.

use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;

pub mod command_palette;
pub mod devices;
pub mod editor;
pub mod grid;
pub mod help;
pub mod logs;
pub mod navigation;
pub mod options;
pub mod saveload;
pub mod splash;

/// A trait defining the core interface for UI components in the application.
///
/// This trait must be implemented by all UI components to ensure consistent behavior
/// across the application. It provides two essential methods:
///
/// - `handle_key_event`: Processes keyboard input and updates component state
/// - `draw`: Renders the component to the terminal screen
///
/// # Type Parameters
///
/// * `Self`: The implementing type must be mutable for handling events
///
/// # Methods
///
/// * `handle_key_event`: Processes keyboard events and returns whether the event was consumed
/// * `draw`: Renders the component to the provided frame within the specified area
///
/// # Examples
///
/// ```rust
/// impl Component for MyComponent {
///     fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
///         // Handle keyboard input
///         Ok(true) // Event was consumed
///     }
///
///     fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
///         // Render component
///     }
/// }
/// ```
pub trait Component {
    /// Handles keyboard input events for the component.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state, mutable to allow state updates
    /// * `key_event` - The keyboard event to process
    ///
    /// # Returns
    ///
    /// * `EyreResult<bool>` - Ok(true) if the event was consumed, Ok(false) if not,
    ///   or an error if processing failed
    fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool>;

    /// Renders the component to the terminal screen.
    ///
    /// This method is responsible for drawing the component's visual representation
    /// within the specified area of the terminal frame. Components should use the
    /// provided frame and area to render their UI elements.
    ///
    /// # Arguments
    ///
    /// * `app` - The application state containing data and settings
    /// * `frame` - The terminal frame to render to
    /// * `area` - The rectangular area within which to render the component
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
    ///     let block = Block::default()
    ///         .title("My Component")
    ///         .borders(Borders::ALL);
    ///     frame.render_widget(block, area);
    /// }
    /// ```
    fn draw(&self, app: &App, frame: &mut ratatui::Frame, area: Rect);
}
