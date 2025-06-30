//! Input handling modules for the grid component.
//!
//! This module provides organized input handling for the grid component,
//! split into logical categories for better maintainability.

pub mod editing;
pub mod navigation;
pub mod prompts;
pub mod selection;

use crate::app::App;
use color_eyre::Result as EyreResult;
use crossterm::event::KeyEvent;

/// Main input handler that delegates to specific handlers based on the current state.
#[derive(Clone)]
pub struct GridInputHandler;

impl GridInputHandler {
    pub fn new() -> Self {
        Self
    }

    /// Handle key events by delegating to appropriate sub-handlers.
    pub fn handle_key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EyreResult<bool> {
        // Check if help is showing first
        if app.interface.components.grid_show_help {
            return navigation::NavigationHandler::handle_help_mode(app, key_event);
        }

        // Handle input prompts first (highest priority)
        if prompts::PromptHandler::is_in_prompt_mode(app) {
            return prompts::PromptHandler::handle_prompt_input(app, key_event);
        }

        // Handle general grid navigation and actions
        // Try each handler in sequence until one handles the input
        if let Ok(handled) = navigation::NavigationHandler::handle_navigation(app, key_event) {
            if handled {
                return Ok(true);
            }
        }

        if let Ok(handled) = editing::EditingHandler::handle_editing(app, key_event) {
            if handled {
                return Ok(true);
            }
        }

        if let Ok(handled) = selection::SelectionHandler::handle_selection(app, key_event) {
            if handled {
                return Ok(true);
            }
        }

        // No handler processed the input
        Ok(false)
    }
}

impl Default for GridInputHandler {
    fn default() -> Self {
        Self::new()
    }
}
